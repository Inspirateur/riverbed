use crate::block::Face;
use crate::render::texture_array::{BlockTextureArray, TextureMap};
use crate::render::voxel_grid_mesh_thread::{
    GridColliderOutput, GridMeshOutput, spawn_grid_mesh_thread,
};
use crate::world::{CHUNK_S1, Chunk, GridChunkPos, VoxelGrid};
use avian3d::prelude::{CenterOfMass, Collider, Dominance, Friction, Mass, RigidBody};
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use crossbeam::channel::{Receiver, unbounded};
use crossbeam_skiplist::SkipMap;
use itertools::Itertools;
use parking_lot::RwLock;
use std::collections::HashMap;

use super::BlockTexState;

pub struct VoxelGridPlugin;

impl Plugin for VoxelGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pull_grid_meshes, pull_grid_colliders).run_if(in_state(BlockTexState::Mapped)),
        );
    }
}

#[derive(Component)]
pub struct GridMeshReceiver(pub Receiver<GridMeshOutput>);

#[derive(Component)]
pub struct GridColliderReceiver(pub Receiver<GridColliderOutput>);

/// Tracks the visual and collider child entities owned by a `VoxelGrid` root,
/// keyed by chunk position. Pull systems use it to update or despawn children
/// when chunk geometry changes.
#[derive(Component, Default)]
pub struct GridChildEntities {
    pub(crate) mesh: HashMap<(GridChunkPos, Face), Entity>,
    pub collider: HashMap<GridChunkPos, Entity>,
}

/// Spawn a movable voxel grid: creates the worker, runs the `build` closure to
/// fill blocks, spawns the rigidbody root, and queues every populated chunk
/// for meshing. Must be called while `BlockTexState::Mapped` is active so the
/// worker captures a populated texture map.
///
/// Colliders for the initial chunks are computed *synchronously* and attached
/// as children at spawn time. This keeps Avian from seeing a `RigidBody::Dynamic`
/// with an empty compound (which produces a "no mass" warning and can NaN
/// the whole physics step). Visual meshes are still computed asynchronously by
/// the worker — they appear a frame or two later, which is fine.
pub fn spawn_voxel_grid(
    commands: &mut Commands,
    texture_map: &TextureMap,
    transform: Transform,
    build: impl FnOnce(&mut VoxelGrid),
) -> Entity {
    let (order_sender, order_receiver) = unbounded::<GridChunkPos>();
    let mut grid = VoxelGrid::new(order_sender.clone());
    build(&mut grid);
    let chunks = grid.chunks.clone();

    // Padding stays default-zero (no neighbour sync). Each chunk's trimesh
    // therefore emits faces on every chunk boundary, making it a *closed
    // manifold* of its own block contents — required by Parry's
    // `mass_properties` to produce a sane volume. The visual cost is
    // duplicate seam surfaces between adjacent grid chunks; sibling
    // colliders of the same compound don't self-collide, so this is purely
    // cosmetic (z-fighting at seams on multi-chunk grids only).
    let initial_colliders: Vec<(GridChunkPos, Collider)> = chunks
        .iter()
        .filter_map(|entry| {
            let chunk_pos = *entry.key();
            let guard = entry.value().read();
            guard
                .create_collider_data()
                .map(|(verts, idx)| (chunk_pos, Collider::trimesh(verts, idx)))
        })
        .collect();

    // Mass + COM from per-block density. Overrides Avian's default
    // (collider-volume × default density), which would treat a stone block
    // and a leaf block as equally heavy.
    let (mass, com) = compute_mass_properties(&chunks);

    let (mesh_receiver, collider_receiver) =
        spawn_grid_mesh_thread(chunks.clone(), texture_map.0.clone(), order_receiver);

    let entity = commands
        .spawn((
            grid,
            RigidBody::Dynamic,
            Friction::new(1.0),
            Dominance(10),
            Mass(mass),
            CenterOfMass(com),
            transform,
            Visibility::default(),
            GridMeshReceiver(mesh_receiver),
            GridColliderReceiver(collider_receiver),
        ))
        .id();

    let mut tracker = GridChildEntities::default();
    for (chunk_pos, collider) in initial_colliders {
        let child = commands
            .spawn((
                collider,
                Transform::from_translation(Vec3::from(chunk_pos)),
            ))
            .id();
        commands.entity(entity).add_child(child);
        tracker.collider.insert(chunk_pos, child);
    }
    commands.entity(entity).insert(tracker);

    for entry in chunks.iter() {
        let _ = order_sender.send(*entry.key());
    }
    entity
}

fn pull_grid_meshes(
    mut commands: Commands,
    mut grids: Query<(Entity, &GridMeshReceiver, &mut GridChildEntities)>,
    mut mesh_query: Query<&mut Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
    block_tex_array: Res<BlockTextureArray>,
) {
    for (grid_entity, recv, mut tracker) in grids.iter_mut() {
        // Coalesce duplicates so older geometry is dropped.
        let received: Vec<_> = recv.0.try_iter().collect();
        for (mesh_opt, chunk_pos, face) in received
            .into_iter()
            .rev()
            .unique_by(|(_, pos, face)| (*pos, *face))
        {
            let key = (chunk_pos, face);
            let Some(mesh) = mesh_opt else {
                if let Some(ent) = tracker.mesh.remove(&key) {
                    commands.entity(ent).despawn();
                }
                continue;
            };
            if let Some(&ent) = tracker.mesh.get(&key) {
                if let Ok(mut handle) = mesh_query.get_mut(ent) {
                    handle.0 = meshes.add(mesh);
                    continue;
                }
            }
            let chunk_aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_S1 as f32));
            let child = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(block_tex_array.0.clone()),
                    Transform::from_translation(Vec3::from(chunk_pos)),
                    NoFrustumCulling,
                    chunk_aabb,
                ))
                .id();
            commands.entity(grid_entity).add_child(child);
            tracker.mesh.insert(key, child);
        }
    }
}

fn pull_grid_colliders(
    mut commands: Commands,
    mut grids: Query<(Entity, &GridColliderReceiver, &mut GridChildEntities)>,
    mut collider_query: Query<&mut Collider>,
) {
    for (grid_entity, recv, mut tracker) in grids.iter_mut() {
        let received: Vec<_> = recv.0.try_iter().collect();
        for (collider_opt, chunk_pos) in received.into_iter().rev().unique_by(|(_, pos)| *pos) {
            let Some(collider) = collider_opt else {
                if let Some(ent) = tracker.collider.remove(&chunk_pos) {
                    commands.entity(ent).despawn();
                }
                continue;
            };
            if let Some(&ent) = tracker.collider.get(&chunk_pos) {
                if let Ok(mut handle) = collider_query.get_mut(ent) {
                    *handle = collider;
                    continue;
                }
            }
            let child = commands
                .spawn((
                    collider,
                    Transform::from_translation(Vec3::from(chunk_pos)),
                ))
                .id();
            commands.entity(grid_entity).add_child(child);
            tracker.collider.insert(chunk_pos, child);
        }
    }
}

/// Walk every populated cell of every chunk, summing density and the
/// density-weighted position. Returns `(mass, centre_of_mass)` in the grid's
/// body-local frame. Centre of mass is `Vec3::ZERO` for an all-air grid;
/// callers should fall back gracefully (Avian only uses CoM if mass > 0).
fn compute_mass_properties(
    chunks: &SkipMap<GridChunkPos, RwLock<Chunk>>,
) -> (f32, Vec3) {
    let mut total_mass: f32 = 0.0;
    let mut moment: Vec3 = Vec3::ZERO;
    for entry in chunks.iter() {
        let chunk_pos = *entry.key();
        let chunk_origin = Vec3::from(chunk_pos);
        let guard = entry.value().read();
        for x in 0..CHUNK_S1 {
            for y in 0..CHUNK_S1 {
                for z in 0..CHUNK_S1 {
                    let block = guard.get(crate::world::ChunkedPos { x, y, z });
                    let d = block.density();
                    if d <= 0.0 {
                        continue;
                    }
                    let cell_centre = chunk_origin
                        + Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                    total_mass += d;
                    moment += cell_centre * d;
                }
            }
        }
    }
    let com = if total_mass > 0.0 {
        moment / total_mass
    } else {
        Vec3::ZERO
    };
    (total_mass, com)
}

