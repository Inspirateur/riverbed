use std::fs;
use std::iter::zip;
use crate::items::{BlockBreakTable, BreakEntry, DropQuantity, Hotbar, Item, Stack};
use crate::render::FpsCam;
use crate::sounds::ItemGet;
use crate::ui::{ControllingPlayer, SelectedHotbarSlot};
use crate::blocks::{Block, BlockPos, Blocks, Realm};
use crate::agents::{TargetBlock, Action, PlayerControlled};
use crate::WorldRng;
use leafwing_input_manager::prelude::*;
use bevy::prelude::*;
use rand::Rng;


pub struct BlockActionPlugin;

impl Plugin for BlockActionPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(json5::from_str::<BlockBreakTable>(&fs::read_to_string("assets/data/block_breaking.json5").unwrap()).unwrap())
			.add_systems(Update, (break_action, target_block, target_block_changed).chain().run_if(in_state(ControllingPlayer)))
			.add_systems(Update, block_outline.run_if(in_state(ControllingPlayer)))
            .add_systems(Update, place_block.run_if(in_state(ControllingPlayer)))
			;
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct BreakingAction {
    pub block_pos: BlockPos,
    pub block: Block,
    pub time_left: f32,
    pub break_entry: BreakEntry,
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

fn target_block(
    mut player: Query<(&mut TargetBlock, &Realm), With<PlayerControlled>>, 
    player_cam: Query<&GlobalTransform, With<FpsCam>>,
    world: Res<Blocks>
) {
    let (mut target_block, realm) = player.single_mut();
    let transform = player_cam.single();
    target_block.0 = world.raycast(
        *realm, 
        transform.translation(), 
        *transform.forward(), 
        TARGET_DIST
    );
}

fn target_block_changed(mut commands: Commands, target_query: Query<(Entity, &BreakingAction, &TargetBlock), (With<BreakingAction>, Changed<TargetBlock>)>) {
    for (player, break_action, target_block_opt) in target_query.iter() {
        if target_block_opt.0.is_none() || break_action.block_pos != target_block_opt.0.as_ref().unwrap().pos {
            commands.entity(player).remove::<BreakingAction>();
        }
    }
}

fn block_outline(mut gizmos: Gizmos, target_block_query: Query<&TargetBlock>) {
    for target_block_opt in target_block_query.iter() {
        if let Some(target_block) = &target_block_opt.0 {
            let pos: Vec3 = target_block.pos.into();
            for (anchor, lines) in zip(EDGES_ANCHORS, EDGES_LINES) {
                let anchor_pos = pos + anchor;
                gizmos.line(anchor_pos, anchor_pos+lines*Vec3::X, Color::BLACK);
                gizmos.line(anchor_pos, anchor_pos+lines*Vec3::Y, Color::BLACK);
                gizmos.line(anchor_pos, anchor_pos+lines*Vec3::Z, Color::BLACK);
            }
        }
    }
}

fn break_action(
    mut commands: Commands,
    world: Res<Blocks>, 
    mut block_action_query: Query<(Entity, &TargetBlock, &mut Hotbar, &ActionState<Action>, Option<&mut BreakingAction>)>,
    selected_slot: Res<SelectedHotbarSlot>,
    block_break_table: Res<BlockBreakTable>,
    time: Res<Time>,
    mut world_rng: ResMut<WorldRng>
) {
    for (player, target_block_opt, mut hotbar, action, opt_breaking) in block_action_query.iter_mut() {
        let Some(mut breaking) = opt_breaking else {
            // No current breaking action, we add one
            if !action.pressed(&Action::Action1) {
                continue;
            }
            let Some(target_block) = &target_block_opt.0 else {
                continue;
            };
            let block = world.get_block(target_block.pos);
            if !block.is_targetable() {
                continue;
            }
            let tool_used = hotbar.0.0[selected_slot.0].item();
            let break_entry = block_break_table.get(tool_used, &block);
            let Some(hardness) = break_entry.hardness else {
                continue;
            };
            commands.entity(player).insert(BreakingAction {
                block_pos: target_block.pos,
                block,
                time_left: hardness, 
                break_entry 
            });
            continue;
        };
        if !action.pressed(&Action::Action1) {
            commands.entity(player).remove::<BreakingAction>();
            continue;
        };
        breaking.time_left -= time.delta_seconds();
        if breaking.time_left > 0. {
            continue;
        }
        let Some(target_block) = &target_block_opt.0 else {
            continue;
        };
        world.set_block(target_block.pos, Block::Air);
        if let Some(drop) = breaking.break_entry.drops {
            let drop_quantity = breaking.break_entry.quantity.as_ref().unwrap_or(&DropQuantity::Fixed(1));
            let quantity = match drop_quantity {
                DropQuantity::Fixed(q) => *q,
                DropQuantity::Range { min, max } => {
                    let rng = &mut world_rng.rng;
                    rng.gen_range(*min..=*max)
                }
            };
            if hotbar.0.try_add(Stack::Some(drop, quantity)).is_none() {
                commands.trigger_targets(ItemGet, player);
            }
        }
    }
}

fn place_block(
    world: Res<Blocks>, 
    mut block_action_query: Query<(&TargetBlock, &mut Hotbar, &ActionState<Action>)>, 
    selected_slot: Res<SelectedHotbarSlot>
) {
    for (target_block_opt, mut hotbar, action) in block_action_query.iter_mut() {
        if !action.just_pressed(&Action::Action2) {
            continue;
        }
        let Some(target_block) = &target_block_opt.0 else {
            continue;
        };
        let pos = target_block.pos+target_block.normal;
        if world.get_block(pos).is_targetable() {
            continue;
        }
        let Stack::Some(Item::Block(block), _) = hotbar.0.0[selected_slot.0].take(1) else {
            continue;
        };
        if !world.set_block_safe(pos, block) {
            // If the block couldn't be added we add it back
            hotbar.0.0[selected_slot.0].try_add(Stack::Some(Item::Block(block), 1));
        }
    }
}
