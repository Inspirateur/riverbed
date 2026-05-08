use crate::Block;
use crate::WorldRng;
use crate::agents::{
    Action, PlayerControlled, TargetBlock, TargetKind,
};
use crate::items::{
    BlockLootTable, DropQuantity, FiringTable, InventoryTrait, Item, LootEntry, Stack,
};
use crate::render::FpsCam;
use crate::sounds::ItemGet;
use crate::ui::{CursorGrabbed, GameUiState, ItemHolder, SelectedHotbarSlot};
use crate::world::{BlockEntities, BlockPos, GridBlockPos, Realm, VoxelGrid, VoxelWorld};
use avian3d::prelude::{Collider, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::RngExt;
use std::fs;
use std::iter::zip;
use std::time::{Duration, Instant};

pub struct BlockHitPlacePlugin;

impl Plugin for BlockHitPlacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlockBreakTable(
            json5::from_str::<BlockLootTable>(
                &fs::read_to_string("assets/data/block_breaking.json5").unwrap(),
            )
            .unwrap(),
        ))
        .insert_resource(BlockHarvestTable(
            json5::from_str::<BlockLootTable>(
                &fs::read_to_string("assets/data/block_harvesting.json5").unwrap(),
            )
            .unwrap(),
        ))
        .insert_resource(
            json5::from_str::<FiringTable>(
                &fs::read_to_string("assets/data/firing.json5").unwrap(),
            )
            .unwrap(),
        )
        .add_message::<BlockDestroyed>()
        .add_systems(
            Update,
            (break_action, target_block, target_block_changed)
                .chain()
                .run_if(in_state(GameUiState::None)),
        )
        .add_systems(Update, block_outline.run_if(in_state(CursorGrabbed)))
        .add_systems(Update, place_block.run_if(in_state(GameUiState::None)))
        .add_systems(Update, renew_block);
    }
}

#[derive(Resource)]
struct BlockBreakTable(BlockLootTable);

#[derive(Resource)]
struct BlockHarvestTable(BlockLootTable);

pub enum BlockActionType {
    Breaking,
    Harvesting,
}

/// The block a `BlockLootAction` is operating on — used to detect target
/// changes and to apply the eventual edit.
#[derive(Debug, Clone, PartialEq)]
pub enum LootTarget {
    World(BlockPos),
    Grid { grid: Entity, pos: GridBlockPos },
}

/// The block a per-block side-effect entity (currently `Renewable`) is bound to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttachedBlock {
    World(BlockPos),
    Grid { grid: Entity, pos: GridBlockPos },
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct BlockLootAction {
    pub target: LootTarget,
    pub block: Block,
    pub action_type: BlockActionType,
    pub time_left: f32,
    pub break_entry: LootEntry,
}

const TARGET_DIST: f32 = 10.;
const EDGES_ANCHORS: [Vec3; 4] = [
    Vec3::ZERO,
    Vec3::new(1., 1., 0.),
    Vec3::new(1., 0., 1.),
    Vec3::new(0., 1., 1.),
];
const EDGES_LINES: [Vec3; 4] = [
    Vec3::ONE,
    Vec3::new(-1., -1., 1.),
    Vec3::new(-1., 1., -1.),
    Vec3::new(1., -1., -1.),
];

#[derive(Component)]
pub struct BlockAttached(pub AttachedBlock);

#[derive(Component)]
pub struct Renewable {
    // TODO: fine for now but it's better if the world has its own clock
    renew_after: Instant,
}

/// Camera-forward raycast: takes the closer of the world's DDA-based hit and
/// Avian's physics raycast (used for movable grids). Only colliders that
/// belong to a `VoxelGrid` ancestor are accepted from the physics side, so
/// world-chunk colliders don't double-up against the DDA result.
fn target_block(
    mut player: Query<(Entity, &mut TargetBlock, &Realm), With<PlayerControlled>>,
    player_cam: Query<&GlobalTransform, With<FpsCam>>,
    world: Res<VoxelWorld>,
    spatial_query: SpatialQuery,
    grids: Query<(Entity, &GlobalTransform, &VoxelGrid)>,
    parents: Query<&ChildOf>,
) {
    let (player_entity, mut target_block, realm) = player.single_mut().unwrap();
    let cam = player_cam.single().unwrap();
    let origin = cam.translation();
    let dir = *cam.forward();

    let world_hit = world.raycast(*realm, origin, dir, TARGET_DIST, true);
    let world_dist = world_hit
        .as_ref()
        .map(|hit| approx_block_distance(hit.pos, origin));

    let grid_hit = (|| -> Option<TargetKind> {
        let dir3 = Dir3::new(dir).ok()?;
        let filter = SpatialQueryFilter::default().with_excluded_entities([player_entity]);
        let hit = spatial_query.cast_ray(origin, dir3, TARGET_DIST, true, &filter)?;
        let (grid_entity, grid_xform) = find_grid_ancestor(hit.entity, &grids, &parents)?;
        // Reject if a world-chunk collider sits in front of the grid hit.
        if let Some(wd) = world_dist {
            if wd < hit.distance {
                return None;
            }
        }
        let world_point = origin + dir * hit.distance;
        let world_inside = world_point - hit.normal * 0.01;
        let grid_inverse = grid_xform.affine().inverse();
        let local = grid_inverse.transform_point3(world_inside);
        let pos = GridBlockPos {
            x: local.x.floor() as i32,
            y: local.y.floor() as i32,
            z: local.z.floor() as i32,
        };
        let normal_local = axis_aligned(grid_inverse.transform_vector3(hit.normal));
        Some(TargetKind::Grid {
            grid: grid_entity,
            pos,
            normal_local,
        })
    })();

    target_block.0 = grid_hit.or_else(|| world_hit.map(TargetKind::World));
}

fn approx_block_distance(pos: BlockPos, origin: Vec3) -> f32 {
    let center = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32) + Vec3::splat(0.5);
    (center - origin).length()
}

/// Snap a (near-axis-aligned) vector to the closest unit cardinal axis. Used
/// to convert a trimesh-face hit normal into a block-face normal suitable for
/// `+pos` placement offsets.
fn axis_aligned(v: Vec3) -> Vec3 {
    let abs = v.abs();
    if abs.x >= abs.y && abs.x >= abs.z {
        Vec3::new(v.x.signum(), 0., 0.)
    } else if abs.y >= abs.z {
        Vec3::new(0., v.y.signum(), 0.)
    } else {
        Vec3::new(0., 0., v.z.signum())
    }
}

fn find_grid_ancestor(
    mut entity: Entity,
    grids: &Query<(Entity, &GlobalTransform, &VoxelGrid)>,
    parents: &Query<&ChildOf>,
) -> Option<(Entity, GlobalTransform)> {
    loop {
        if let Ok((e, xform, _)) = grids.get(entity) {
            return Some((e, *xform));
        }
        let child_of = parents.get(entity).ok()?;
        entity = child_of.parent();
    }
}

fn target_block_changed(
    mut commands: Commands,
    target_query: Query<
        (Entity, &BlockLootAction, &TargetBlock),
        (With<BlockLootAction>, Changed<TargetBlock>),
    >,
) {
    for (player, break_action, target_block_opt) in target_query.iter() {
        let still_valid = match (&target_block_opt.0, &break_action.target) {
            (Some(TargetKind::World(hit)), LootTarget::World(pos)) => hit.pos == *pos,
            (Some(TargetKind::Grid { grid: g1, pos: p1, .. }), LootTarget::Grid { grid: g2, pos: p2 }) => {
                g1 == g2 && p1 == p2
            }
            _ => false,
        };
        if !still_valid {
            commands.entity(player).remove::<BlockLootAction>();
        }
    }
}

fn block_outline(
    mut gizmos: Gizmos,
    target_block_query: Query<&TargetBlock>,
    grids: Query<&GlobalTransform, With<VoxelGrid>>,
) {
    for target_block_opt in target_block_query.iter() {
        match &target_block_opt.0 {
            Some(TargetKind::World(hit)) => {
                let pos: Vec3 = hit.pos.into();
                draw_block_outline(&mut gizmos, pos, Quat::IDENTITY);
            }
            Some(TargetKind::Grid { grid, pos, .. }) => {
                let Ok(xform) = grids.get(*grid) else {
                    continue;
                };
                let local = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
                let (_, rot, _) = xform.to_scale_rotation_translation();
                draw_block_outline(&mut gizmos, xform.transform_point(local), rot);
            }
            None => {}
        }
    }
}

fn draw_block_outline(gizmos: &mut Gizmos, origin: Vec3, rotation: Quat) {
    for (anchor, lines) in zip(EDGES_ANCHORS, EDGES_LINES) {
        let anchor_pos = origin + rotation * anchor;
        gizmos.line(
            anchor_pos,
            anchor_pos + rotation * (lines * Vec3::X),
            Color::BLACK,
        );
        gizmos.line(
            anchor_pos,
            anchor_pos + rotation * (lines * Vec3::Y),
            Color::BLACK,
        );
        gizmos.line(
            anchor_pos,
            anchor_pos + rotation * (lines * Vec3::Z),
            Color::BLACK,
        );
    }
}

fn break_action(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    grids: Query<&VoxelGrid>,
    mut block_action_query: Query<(
        Entity,
        &TargetBlock,
        &mut ItemHolder,
        &ActionState<Action>,
        Option<&mut BlockLootAction>,
    )>,
    selected_slot: Res<SelectedHotbarSlot>,
    block_break_table: Res<BlockBreakTable>,
    block_harvest_table: Res<BlockHarvestTable>,
    time: Res<Time>,
    mut col_entities: ResMut<BlockEntities>,
    mut world_rng: ResMut<WorldRng>,
    block_entt_query: Query<(Entity, &BlockAttached)>,
    mut destroyed_writer: MessageWriter<BlockDestroyed>,
) {
    for (player, target_block_opt, mut hotbar, action, opt_looting) in block_action_query.iter_mut()
    {
        let Some(mut looting) = opt_looting else {
            // No current looting action — try to start one
            let action_type = if action.pressed(&Action::Hit) {
                BlockActionType::Breaking
            } else if action.pressed(&Action::Modify) {
                BlockActionType::Harvesting
            } else {
                continue;
            };
            let Some(target) = &target_block_opt.0 else {
                continue;
            };
            let (block, loot_target) = match target {
                TargetKind::World(hit) => (world.get_block(hit.pos), LootTarget::World(hit.pos)),
                TargetKind::Grid { grid, pos, .. } => {
                    let Ok(g) = grids.get(*grid) else { continue };
                    (
                        g.get_block(*pos),
                        LootTarget::Grid {
                            grid: *grid,
                            pos: *pos,
                        },
                    )
                }
            };
            if !block.is_targetable() {
                continue;
            }
            let tool_used = hotbar.get(selected_slot.0).item();
            let break_entry = match action_type {
                BlockActionType::Breaking => block_break_table.0.get(tool_used, &block),
                BlockActionType::Harvesting => block_harvest_table.0.get(tool_used, &block),
            };
            let Some(hardness) = break_entry.hardness else {
                continue;
            };
            commands.entity(player).insert(BlockLootAction {
                target: loot_target,
                block,
                action_type,
                time_left: hardness,
                break_entry,
            });
            continue;
        };
        // There's an existing action — count it down
        if !action.pressed(&match looting.action_type {
            BlockActionType::Breaking => Action::Hit,
            BlockActionType::Harvesting => Action::Modify,
        }) {
            commands.entity(player).remove::<BlockLootAction>();
            continue;
        };
        looting.time_left -= time.delta_secs();
        if looting.time_left > 0. {
            continue;
        }
        // Action completed — apply the edit
        match (&looting.action_type, &looting.target) {
            (BlockActionType::Breaking, LootTarget::World(pos)) => {
                world.set_block(*pos, Block::Air);
                if let Some(entity) = col_entities.get(pos) {
                    if let Ok((_, attached)) = block_entt_query.get(entity) {
                        if matches!(attached.0, AttachedBlock::World(p) if p == *pos) {
                            commands.entity(entity).despawn();
                        }
                    }
                }
                destroyed_writer.write(BlockDestroyed::World(*pos));
            }
            (BlockActionType::Breaking, LootTarget::Grid { grid, pos }) => {
                if let Ok(g) = grids.get(*grid) {
                    g.set_block(*pos, Block::Air);
                }
                // Despawn any side-effect entity (e.g. a pending renewable
                // timer) tied to this grid block. World blocks use the
                // `BlockEntities` map; grids iterate since few exist.
                for (entity, attached) in block_entt_query.iter() {
                    if let AttachedBlock::Grid {
                        grid: g2,
                        pos: p2,
                    } = attached.0
                    {
                        if g2 == *grid && p2 == *pos {
                            commands.entity(entity).despawn();
                        }
                    }
                }
                destroyed_writer.write(BlockDestroyed::Grid {
                    grid: *grid,
                    pos: *pos,
                });
            }
            (BlockActionType::Harvesting, LootTarget::World(pos)) => {
                let depleted = world.get_block(*pos).depleted();
                world.set_block(*pos, depleted);
                if let Some(renewal_minutes) = depleted.renewal_minutes() {
                    let renew_entt = commands
                        .spawn((
                            Renewable {
                                renew_after: Instant::now()
                                    .checked_add(Duration::from_secs(renewal_minutes as u64))
                                    .unwrap(),
                            },
                            BlockAttached(AttachedBlock::World(*pos)),
                        ))
                        .id();
                    col_entities.add(pos, renew_entt);
                }
            }
            (BlockActionType::Harvesting, LootTarget::Grid { grid, pos }) => {
                if let Ok(g) = grids.get(*grid) {
                    let depleted = g.get_block(*pos).depleted();
                    g.set_block(*pos, depleted);
                    if let Some(renewal_minutes) = depleted.renewal_minutes() {
                        commands.spawn((
                            Renewable {
                                renew_after: Instant::now()
                                    .checked_add(Duration::from_secs(renewal_minutes as u64))
                                    .unwrap(),
                            },
                            BlockAttached(AttachedBlock::Grid {
                                grid: *grid,
                                pos: *pos,
                            }),
                        ));
                    }
                }
            }
        }
        if let Some(drop) = looting.break_entry.drops {
            let drop_quantity = looting
                .break_entry
                .quantity
                .as_ref()
                .unwrap_or(&DropQuantity::Fixed(1));
            let quantity = match drop_quantity {
                DropQuantity::Fixed(q) => *q,
                DropQuantity::Range { min, max } => {
                    let rng = &mut world_rng.rng;
                    rng.random_range(*min..=*max)
                }
            };
            let ItemHolder::Inventory(ref mut hotbar) = *hotbar else {
                continue;
            };
            if hotbar.try_add(Stack::Some(drop, quantity)).is_none() {
                commands.trigger(ItemGet { entity: player });
            }
        }
        commands.entity(player).remove::<BlockLootAction>();
    }
}

#[derive(Event)]
pub struct BlockPlaced(pub BlockPos);

/// Fired whenever a solid block becomes Air via player action — used by the
/// disconnected-component detector to decide when to split a piece off into
/// its own movable grid.
#[derive(Message)]
pub enum BlockDestroyed {
    World(BlockPos),
    Grid { grid: Entity, pos: GridBlockPos },
}

fn place_block(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    grids: Query<&VoxelGrid>,
    grid_xforms: Query<(Entity, &GlobalTransform, &VoxelGrid)>,
    mut block_action_query: Query<(
        Entity,
        &TargetBlock,
        &mut ItemHolder,
        &ActionState<Action>,
    )>,
    selected_slot: Res<SelectedHotbarSlot>,
    spatial_query: SpatialQuery,
) {
    for (player_entity, target_block_opt, mut hotbar, action) in block_action_query.iter_mut() {
        if !action.just_pressed(&Action::Modify) {
            continue;
        }
        let Some(target) = &target_block_opt.0 else {
            continue;
        };
        let block = match hotbar.get_mut(selected_slot.0).take(1) {
            Stack::Some(Item::Block(block), _) => block,
            other => {
                hotbar.get_mut(selected_slot.0).try_add(other);
                continue;
            }
        };
        let placed = match target {
            TargetKind::World(hit) => {
                let pos = hit.pos + hit.normal;
                if !world.get_block(pos).is_traversable() {
                    false
                } else {
                    let center =
                        Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32) + Vec3::splat(0.5);
                    if placement_overlaps(
                        &spatial_query,
                        center,
                        Quat::IDENTITY,
                        &[player_entity],
                    ) {
                        false
                    } else if world.set_block_safe(pos, block) {
                        commands.trigger(BlockPlaced(pos));
                        true
                    } else {
                        false
                    }
                }
            }
            TargetKind::Grid {
                grid,
                pos,
                normal_local,
                ..
            } => {
                let Ok(g) = grids.get(*grid) else { continue };
                let new_pos = GridBlockPos {
                    x: pos.x + normal_local.x as i32,
                    y: pos.y + normal_local.y as i32,
                    z: pos.z + normal_local.z as i32,
                };
                if !g.get_block(new_pos).is_traversable() {
                    false
                } else {
                    let Ok((_, grid_xform, _)) = grid_xforms.get(*grid) else {
                        continue;
                    };
                    let (_, rotation, _) = grid_xform.to_scale_rotation_translation();
                    let center = grid_xform.transform_point(
                        Vec3::new(new_pos.x as f32, new_pos.y as f32, new_pos.z as f32)
                            + Vec3::splat(0.5),
                    );
                    if placement_overlaps(
                        &spatial_query,
                        center,
                        rotation,
                        &[player_entity],
                    ) {
                        false
                    } else {
                        g.set_block(new_pos, block);
                        true
                    }
                }
            }
        };
        if !placed {
            hotbar
                .get_mut(selected_slot.0)
                .try_add(Stack::Some(Item::Block(block), 1));
        }
    }
}

/// True if a 1m³ block at `(center, rotation)` would overlap an existing
/// collider. The probe is 0.95m so neighbouring blocks (whose trimesh faces
/// sit on the placement cell's boundaries) don't false-positive as touching
/// contacts; rotated grids exceed that 0.025m clearance whenever they
/// genuinely overlap.
fn placement_overlaps(
    spatial_query: &SpatialQuery,
    center: Vec3,
    rotation: Quat,
    excluded: &[Entity],
) -> bool {
    let probe = Collider::cuboid(0.95, 0.95, 0.95);
    let filter =
        SpatialQueryFilter::default().with_excluded_entities(excluded.iter().copied());
    !spatial_query
        .shape_intersections(&probe, center, rotation, &filter)
        .is_empty()
}

fn renew_block(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    grids: Query<&VoxelGrid>,
    renewables: Query<(Entity, &Renewable, &BlockAttached)>,
) {
    let now = Instant::now();
    for (entity, renewable, attached) in renewables.iter() {
        if now < renewable.renew_after {
            continue;
        }
        match attached.0 {
            AttachedBlock::World(pos) => {
                world.set_block(pos, world.get_block(pos).renewed());
            }
            AttachedBlock::Grid { grid, pos } => {
                // If the grid was despawned, the renewable is silently
                // dropped when we despawn its entity below.
                if let Ok(g) = grids.get(grid) {
                    g.set_block(pos, g.get_block(pos).renewed());
                }
            }
        }
        commands.entity(entity).despawn();
    }
}
