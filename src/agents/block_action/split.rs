use crate::Block;
use crate::agents::block_action::BlockDestroyed;
use crate::render::{ChunkColliderEntities, GridChildEntities, TextureMap, spawn_voxel_grid};
use crate::world::{
    BlockPos, Chunk, ChunkPos, ChunkedPos, GridBlockPos, GridChunkPos, Realm, VoxelGrid, VoxelWorld,
};
use avian3d::prelude::{AngularVelocity, Collider, LinearVelocity};
use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender, unbounded};
use crossbeam_skiplist::SkipMap;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::thread;

/// Maximum surface cells visited across all seeds in the connectivity BFS.
/// Surface-only expansion means this caps the total *boundary* of the
/// candidate component; the actual block count of a splittable piece can be
/// much higher (a 20³ solid is ~2400 surface cells).
const SPLIT_BFS_BUDGET: usize = 4096;

/// Cap on the dense flood-fill that collects interior cells once a split has
/// been confirmed. Surface BFS only touches the manifold; interior cells are
/// reached via this pass.
const FILL_COMPONENT_BUDGET: usize = 65536;


/// Resource shared between the BFS spawner and applier — async tasks send
/// completed split outputs here for the main thread to consume.
#[derive(Resource)]
pub struct SplitChannel {
    sender: Sender<SplitOutput>,
    receiver: Receiver<SplitOutput>,
}

impl Default for SplitChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

enum SplitOutput {
    World {
        positions: HashSet<(i32, i32, i32)>,
        realm: Realm,
    },
    Grid {
        source: Entity,
        transform: Transform,
        linear_velocity: LinearVelocity,
        angular_velocity: AngularVelocity,
        positions: HashSet<(i32, i32, i32)>,
    },
}

/// Per-frame system: turns each `BlockDestroyed` event into a BFS job on a
/// fresh OS thread. We use `std::thread::spawn` rather than Bevy's
/// `AsyncComputeTaskPool` because the BFS body is CPU-bound with no `.await`
/// points and would otherwise contend with the chunk mesh worker (which
/// blocks on a channel `recv` inside its own pool task) for the pool's
/// limited threads, looking like a freeze when both run at once.
pub fn schedule_split_jobs(
    mut events: MessageReader<BlockDestroyed>,
    world: Res<VoxelWorld>,
    grids: Query<(
        &GlobalTransform,
        &VoxelGrid,
        Option<&LinearVelocity>,
        Option<&AngularVelocity>,
    )>,
    channel: Res<SplitChannel>,
) {
    for event in events.read() {
        match event {
            BlockDestroyed::World(pos) => {
                let chunks = world.chunks.clone();
                let realm = pos.realm;
                let dest = (pos.x, pos.y, pos.z);
                let sender = channel.sender.clone();
                thread::spawn(move || {
                    let reader = WorldReader { chunks, realm };
                    if let Some(positions) = run_split_bfs(&reader, dest) {
                        let _ = sender.send(SplitOutput::World { positions, realm });
                    }
                });
            }
            BlockDestroyed::Grid { grid, pos } => {
                let Ok((xform, g, lin, ang)) = grids.get(*grid) else {
                    continue;
                };
                let chunks = g.chunks.clone();
                let dest = (pos.x, pos.y, pos.z);
                let source = *grid;
                let transform = xform.compute_transform();
                let linear_velocity = lin.copied().unwrap_or_default();
                let angular_velocity = ang.copied().unwrap_or_default();
                let sender = channel.sender.clone();
                thread::spawn(move || {
                    let reader = GridReader { chunks };
                    if let Some(positions) = run_split_bfs(&reader, dest) {
                        let _ = sender.send(SplitOutput::Grid {
                            source,
                            transform,
                            linear_velocity,
                            angular_velocity,
                            positions,
                        });
                    }
                });
            }
        }
    }
}

/// Drains finished split tasks and applies their results on the main thread.
/// Re-verifies that every block in the proposed component is still solid —
/// the player or another split may have changed the world between the BFS
/// snapshot and the apply, and we don't want to clone air blocks into a grid.
pub fn apply_split_outputs(
    mut commands: Commands,
    channel: Res<SplitChannel>,
    world: Res<VoxelWorld>,
    mut grid_data: Query<(&VoxelGrid, &mut GridChildEntities)>,
    mut world_collider_ents: ResMut<ChunkColliderEntities>,
    texture_map: Res<TextureMap>,
) {
    while let Ok(output) = channel.receiver.try_recv() {
        match output {
            SplitOutput::World { positions, realm } => {
                let valid: Vec<(BlockPos, Block)> = positions
                    .iter()
                    .map(|&(x, y, z)| BlockPos { x, y, z, realm })
                    .filter_map(|p| {
                        let b = world.get_block(p);
                        (!b.is_traversable()).then_some((p, b))
                    })
                    .collect();
                if valid.is_empty() {
                    continue;
                }
                for (p, _) in &valid {
                    world.set_block(*p, Block::Air);
                }
                rebuild_world_chunk_colliders(
                    &mut commands,
                    &world,
                    &mut world_collider_ents,
                    valid.iter().map(|(p, _)| *p),
                );
                // Anchor at the bounding-box minimum so block coords end up
                // non-negative; for any component fitting in CHUNK_S1 cells
                // along each axis this lands the whole piece in one grid
                // chunk, giving Parry a closed-manifold trimesh per child.
                let min_x = valid.iter().map(|(p, _)| p.x).min().unwrap();
                let min_y = valid.iter().map(|(p, _)| p.y).min().unwrap();
                let min_z = valid.iter().map(|(p, _)| p.z).min().unwrap();
                let anchor = Transform::from_xyz(min_x as f32, min_y as f32, min_z as f32);
                spawn_voxel_grid(&mut commands, &texture_map, anchor, |grid| {
                    for (p, block) in valid {
                        grid.set_block(
                            GridBlockPos {
                                x: p.x - min_x,
                                y: p.y - min_y,
                                z: p.z - min_z,
                            },
                            block,
                        );
                    }
                });
            }
            SplitOutput::Grid {
                source,
                transform,
                linear_velocity,
                angular_velocity,
                positions,
            } => {
                let Ok((g, mut tracker)) = grid_data.get_mut(source) else {
                    continue;
                };
                let valid: Vec<(GridBlockPos, Block)> = positions
                    .iter()
                    .map(|&(x, y, z)| GridBlockPos { x, y, z })
                    .filter_map(|p| {
                        let b = g.get_block(p);
                        (!b.is_traversable()).then_some((p, b))
                    })
                    .collect();
                if valid.is_empty() {
                    continue;
                }
                for (p, _) in &valid {
                    g.set_block(*p, Block::Air);
                }
                rebuild_grid_chunk_colliders(
                    &mut commands,
                    g,
                    &mut tracker,
                    valid.iter().map(|(p, _)| *p),
                );
                if tracker.collider.is_empty() {
                    commands.entity(source).despawn();
                }
                let new_entity =
                    spawn_voxel_grid(&mut commands, &texture_map, transform, |grid| {
                        for (p, block) in valid {
                            grid.set_block(p, block);
                        }
                    });
                commands
                    .entity(new_entity)
                    .insert((linear_velocity, angular_velocity));
            }
        }
    }
}

/// Rebuilds the trimesh collider for every world chunk that lost a block in
/// this split. Without this, the world's static collider still claims the
/// space the new dynamic grid is about to occupy, and Avian sees deeply-
/// penetrating dynamic-vs-static contacts on every cell of the split → solver
/// stalls. The async mesh worker would update these eventually, but not
/// before the next physics step.
fn rebuild_world_chunk_colliders(
    commands: &mut Commands,
    world: &VoxelWorld,
    collider_ents: &mut ChunkColliderEntities,
    blocks: impl IntoIterator<Item = BlockPos>,
) {
    let affected: HashSet<ChunkPos> = blocks
        .into_iter()
        .map(|p| <(ChunkPos, ChunkedPos)>::from(p).0)
        .collect();
    for chunk_pos in affected {
        let Some(entry) = world.chunks.get(&chunk_pos) else {
            continue;
        };
        let new_collider = entry
            .value()
            .read()
            .create_collider_data()
            .map(|(v, i)| Collider::trimesh(v, i));
        match (collider_ents.0.get(&chunk_pos).copied(), new_collider) {
            (Some(ent), Some(c)) => {
                commands.entity(ent).insert(c);
            }
            (Some(ent), None) => {
                commands.entity(ent).despawn();
                collider_ents.0.remove(&chunk_pos);
            }
            (None, _) => {
                // No existing collider entity to update — chunk had no surface
                // before the split (unusual). The async worker will spawn one
                // if the new state warrants it.
            }
        }
    }
}

/// Source-grid analogue of `rebuild_world_chunk_colliders`. Without this, the
/// source's existing per-chunk colliders overlap the new grid 1:1 in world
/// space until the async worker catches up.
fn rebuild_grid_chunk_colliders(
    commands: &mut Commands,
    grid: &VoxelGrid,
    tracker: &mut GridChildEntities,
    blocks: impl IntoIterator<Item = GridBlockPos>,
) {
    let affected: HashSet<GridChunkPos> = blocks
        .into_iter()
        .map(|p| <(GridChunkPos, ChunkedPos)>::from(p).0)
        .collect();
    for chunk_pos in affected {
        let Some(entry) = grid.chunks.get(&chunk_pos) else {
            continue;
        };
        let new_collider = entry
            .value()
            .read()
            .create_collider_data()
            .map(|(v, i)| Collider::trimesh(v, i));
        match (tracker.collider.get(&chunk_pos).copied(), new_collider) {
            (Some(ent), Some(c)) => {
                commands.entity(ent).insert(c);
            }
            (Some(ent), None) => {
                commands.entity(ent).despawn();
                tracker.collider.remove(&chunk_pos);
            }
            (None, _) => {}
        }
    }
}

/// Reads voxel data through an Arc-cloneable storage handle so the BFS can
/// run on a worker thread without the main `VoxelWorld` / `VoxelGrid`
/// resources.
///
/// `solidity` returns `Some(true)` for a known-solid cell, `Some(false)` for a
/// known-air cell, and `None` if the cell is *unknown* — this only happens
/// for the world, where chunks load lazily and unloaded chunks could contain
/// either solid or air. Encountering `None` aborts the BFS, because we can't
/// rule out that a "bounded" component actually extends into an unloaded
/// chunk and is still connected to the rest of the terrain.
trait VoxelReader: Send + Sync {
    fn solidity(&self, p: (i32, i32, i32)) -> Option<bool>;
}

struct WorldReader {
    chunks: Arc<SkipMap<ChunkPos, RwLock<Chunk>>>,
    realm: Realm,
}

impl VoxelReader for WorldReader {
    fn solidity(&self, p: (i32, i32, i32)) -> Option<bool> {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(BlockPos {
            x: p.0,
            y: p.1,
            z: p.2,
            realm: self.realm,
        });
        let entry = self.chunks.get(&chunk_pos)?;
        Some(!entry.value().read().get(chunked_pos).is_traversable())
    }
}

struct GridReader {
    chunks: Arc<SkipMap<GridChunkPos, RwLock<Chunk>>>,
}

impl VoxelReader for GridReader {
    fn solidity(&self, p: (i32, i32, i32)) -> Option<bool> {
        let (chunk_pos, chunked_pos) = <(GridChunkPos, ChunkedPos)>::from(GridBlockPos {
            x: p.0,
            y: p.1,
            z: p.2,
        });
        // Grid chunks are self-contained: an absent chunk means definitely
        // air, not "unknown". The grid has no concept of streamed loading.
        Some(
            self.chunks
                .get(&chunk_pos)
                .map(|e| !e.value().read().get(chunked_pos).is_traversable())
                .unwrap_or(false),
        )
    }
}

/// Top-level BFS driver. Seeds the BFS from every solid 26-connected neighbour
/// of the destroyed cell — face, edge, *and* corner — because connectivity in
/// this game is 26-conn, so any of them could have been the only path between
/// two pieces. Returns the smaller disconnected component if one fully closes
/// within budget, `None` otherwise (no split, or unable to verify).
///
/// The surface-only BFS only verifies *connectivity*; it never visits cells
/// surrounded entirely by other solid cells. Once we have a confirmed split
/// component (its surface), `fill_component` flood-fills inwards to collect
/// the interior so the entire mass moves together.
fn run_split_bfs<R: VoxelReader>(
    reader: &R,
    destroyed: (i32, i32, i32),
) -> Option<HashSet<(i32, i32, i32)>> {
    let starts: Vec<(i32, i32, i32)> = neighbors_26()
        .map(|(dx, dy, dz)| (destroyed.0 + dx, destroyed.1 + dy, destroyed.2 + dz))
        .filter(|p| reader.solidity(*p) == Some(true))
        .collect();
    if starts.len() < 2 {
        return None;
    }
    let surface = bfs_multi(reader, &starts, SPLIT_BFS_BUDGET)?;
    fill_component(reader, surface)
}

/// Flood-fill from the surface set through every 26-connected solid cell
/// (no surface filter) to capture interior blocks. Returns the union.
fn fill_component<R: VoxelReader>(
    reader: &R,
    surface: HashSet<(i32, i32, i32)>,
) -> Option<HashSet<(i32, i32, i32)>> {
    let mut visited = surface;
    let mut queue: VecDeque<(i32, i32, i32)> = visited.iter().copied().collect();
    while let Some(p) = queue.pop_front() {
        for offset in neighbors_26() {
            let q = (p.0 + offset.0, p.1 + offset.1, p.2 + offset.2);
            if visited.contains(&q) {
                continue;
            }
            // `?` aborts the whole split if we somehow leak into unloaded
            // territory (surface should already enclose the mass, so this
            // is defensive).
            if !reader.solidity(q)? {
                continue;
            }
            if visited.len() >= FILL_COMPONENT_BUDGET {
                return None;
            }
            visited.insert(q);
            queue.push_back(q);
        }
    }
    Some(visited)
}

/// Multi-seed BFS: one frontier per face-neighbour, expanding through 26-
/// connected solid surface cells. Frontiers union-find together when they
/// visit a common cell. The first group whose frontiers all empty before
/// merging into a single universe is returned as the split component.
///
/// Surface-only expansion: only solid blocks with at least one 26-air-
/// neighbour are added to a frontier. Interior cells of a solid mass are
/// skipped, which keeps the budget useful on dense bodies.
fn bfs_multi<R: VoxelReader>(
    reader: &R,
    starts: &[(i32, i32, i32)],
    budget: usize,
) -> Option<HashSet<(i32, i32, i32)>> {
    let n = starts.len();
    let mut frontiers: Vec<VecDeque<(i32, i32, i32)>> =
        starts.iter().map(|p| VecDeque::from([*p])).collect();
    let mut owner: HashMap<(i32, i32, i32), usize> = HashMap::new();
    let mut parent: Vec<usize> = (0..n).collect();
    for (i, p) in starts.iter().enumerate() {
        owner.insert(*p, i);
    }
    let mut total_expanded = n;

    loop {
        let mut any_active = false;
        for i in 0..n {
            let Some(p) = frontiers[i].pop_front() else {
                continue;
            };
            any_active = true;
            for offset in neighbors_26() {
                let q = (p.0 + offset.0, p.1 + offset.1, p.2 + offset.2);
                // Short-circuit on already-visited cells: skip the expensive
                // `solidity` + `is_surface` reads (each `is_surface` does up
                // to 26 chunk lookups) and just union the BFSes.
                if let Some(&prev_i) = owner.get(&q) {
                    let r1 = uf_find(&mut parent, prev_i);
                    let r2 = uf_find(&mut parent, i);
                    if r1 != r2 {
                        parent[r1] = r2;
                    }
                    continue;
                }
                // `?` aborts the whole BFS if `q` is in unknown territory
                // (unloaded chunk) — we can't trust a "bounded" verdict if we
                // never observed half the boundary.
                if !reader.solidity(q)? {
                    continue;
                }
                if !is_surface(reader, q)? {
                    continue;
                }
                owner.insert(q, i);
                frontiers[i].push_back(q);
                total_expanded += 1;
                if total_expanded >= budget {
                    return None;
                }
            }
        }

        // Group frontiers by union-find root and look for any group that's
        // now fully exhausted while at least one other group is still
        // distinct. That group is a closed disconnected component.
        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let r = uf_find(&mut parent, i);
            groups.entry(r).or_default().push(i);
        }
        if groups.len() == 1 {
            // Every seed has been unioned into the same component, so the
            // destroyed cell wasn't a bridge — no split is possible regardless
            // of how much further we'd expand. This is the fast path for
            // mining a block in the middle of a connected mass.
            return None;
        }
        for (root, members) in &groups {
            if members.iter().all(|&i| frontiers[i].is_empty()) {
                let target_root = *root;
                let component: HashSet<_> = owner
                    .iter()
                    .filter(|(_, i)| uf_find(&mut parent, **i) == target_root)
                    .map(|(p, _)| *p)
                    .collect();
                return Some(component);
            }
        }

        if !any_active {
            // All frontiers empty but multiple groups distinct — every group
            // is a closed component. Pick the smallest as the split target.
            let mut best: Option<(usize, usize)> = None;
            for (root, _) in &groups {
                let target_root = *root;
                let size = owner
                    .iter()
                    .filter(|(_, i)| uf_find(&mut parent, **i) == target_root)
                    .count();
                if best.map(|(_, s)| size < s).unwrap_or(true) {
                    best = Some((target_root, size));
                }
            }
            let (root, _) = best?;
            let component: HashSet<_> = owner
                .iter()
                .filter(|(_, i)| uf_find(&mut parent, **i) == root)
                .map(|(p, _)| *p)
                .collect();
            return Some(component);
        }
    }
}

/// Path-compressing union-find lookup.
fn uf_find(parent: &mut [usize], i: usize) -> usize {
    let mut root = i;
    while parent[root] != root {
        root = parent[root];
    }
    let mut cur = i;
    while parent[cur] != root {
        let next = parent[cur];
        parent[cur] = root;
        cur = next;
    }
    root
}

/// `Some(true)` if at least one of `p`'s 26 surrounding cells is non-solid.
/// `None` if any neighbour is in an unloaded (unknown) cell — we can't tell
/// whether `p` is on the surface or in the interior, so the caller must abort.
/// Used to restrict BFS expansion to the surface manifold of the source mass.
fn is_surface<R: VoxelReader>(reader: &R, p: (i32, i32, i32)) -> Option<bool> {
    for offset in neighbors_26() {
        if !reader.solidity((p.0 + offset.0, p.1 + offset.1, p.2 + offset.2))? {
            return Some(true);
        }
    }
    Some(false)
}

fn neighbors_26() -> impl Iterator<Item = (i32, i32, i32)> {
    (-1i32..=1).flat_map(|dx| {
        (-1i32..=1).flat_map(move |dy| {
            (-1i32..=1).filter_map(move |dz| {
                if dx == 0 && dy == 0 && dz == 0 {
                    None
                } else {
                    Some((dx, dy, dz))
                }
            })
        })
    })
}
