use std::iter::zip;
use crate::GameState;
use crate::blocks::{Blocks, Block, Realm};
use crate::agents::{TargetBlock, Action, PlayerControlled};
use super::camera::FpsCam;
use leafwing_input_manager::prelude::*;
use bevy::prelude::*;

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

pub fn target_block(
    mut player: Query<(&mut TargetBlock, &Realm), With<PlayerControlled>>, 
    player_cam: Query<&GlobalTransform, With<FpsCam>>,
    world: Res<Blocks>
) {
    let (mut target_block, realm) = player.single_mut();
    let transform = player_cam.single();
    target_block.0 = world.raycast(
        *realm, 
        transform.translation(), 
        transform.forward(), 
        TARGET_DIST
    );
}

pub fn block_outline(mut gizmos: Gizmos, target_block_query: Query<&TargetBlock>) {
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

pub fn break_block(world: Res<Blocks>, block_action_query: Query<(&TargetBlock, &ActionState<Action>)>) {
    for (target_block_opt, action) in block_action_query.iter() {
        if action.just_pressed(&Action::Action1) {
            if let Some(target_block) = &target_block_opt.0 {
                world.set_block(target_block.pos, Block::Air);
            }    
        }
    }
}

pub fn place_block(world: Res<Blocks>, block_action_query: Query<(&TargetBlock, &ActionState<Action>)>) {
    for (target_block_opt, action) in block_action_query.iter() {
        if action.just_pressed(&Action::Action2) {
            if let Some(target_block) = &target_block_opt.0 {
                world.set_block_safe(target_block.pos+target_block.normal, Block::GrassBlock);
            }
        }
    }   
}

pub struct BlockActionPlugin;

impl Plugin for BlockActionPlugin {
    fn build(&self, app: &mut App) {
        app
			.add_systems(Update, target_block)
			.add_systems(Update, block_outline)
            .add_systems(Update, break_block.run_if(in_state(GameState::Game)))
            .add_systems(Update, place_block.run_if(in_state(GameState::Game)))
			;
    }
}