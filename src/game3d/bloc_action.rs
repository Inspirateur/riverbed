use std::iter::zip;
use crate::GameState;
use crate::blocs::{Blocs, Bloc, Realm};
use crate::agents::{TargetBloc, Action, PlayerControlled};
use super::camera::FpsCam;
use leafwing_input_manager::prelude::*;
use bevy::prelude::*;

const TARGET_DIST: f32 = 8.;
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

pub fn target_bloc(
    mut player: Query<(&mut TargetBloc, &Realm), With<PlayerControlled>>, 
    player_cam: Query<&GlobalTransform, With<FpsCam>>,
    world: Res<Blocs>
) {
    let (mut target_bloc, realm) = player.single_mut();
    let transform = player_cam.single();
    target_bloc.0 = world.raycast(
        *realm, 
        transform.translation(), 
        transform.forward(), 
        TARGET_DIST
    );
}

pub fn bloc_outline(mut gizmos: Gizmos, target_bloc_query: Query<&TargetBloc>) {
    for target_bloc_opt in target_bloc_query.iter() {
        if let Some(target_bloc) = &target_bloc_opt.0 {
            let pos: Vec3 = target_bloc.pos.into();
            for (anchor, lines) in zip(EDGES_ANCHORS, EDGES_LINES) {
                let anchor_pos = pos + anchor;
                gizmos.line(anchor_pos, anchor_pos+lines*Vec3::X, Color::BLACK);
                gizmos.line(anchor_pos, anchor_pos+lines*Vec3::Y, Color::BLACK);
                gizmos.line(anchor_pos, anchor_pos+lines*Vec3::Z, Color::BLACK);
            }
        }
    }
}

pub fn break_bloc(world: Res<Blocs>, bloc_action_query: Query<(&TargetBloc, &ActionState<Action>)>) {
    for (target_bloc_opt, action) in bloc_action_query.iter() {
        if action.just_pressed(&Action::Action1) {
            if let Some(target_bloc) = &target_bloc_opt.0 {
                world.set_bloc(target_bloc.pos, Bloc::Air);
            }    
        }
    }
}

pub fn place_bloc(world: Res<Blocs>, bloc_action_query: Query<(&TargetBloc, &ActionState<Action>)>) {
    for (target_bloc_opt, action) in bloc_action_query.iter() {
        if action.just_pressed(&Action::Action2) {
            if let Some(target_bloc) = &target_bloc_opt.0 {
                world.set_bloc_safe(target_bloc.pos+target_bloc.normal, Bloc::Stone);
            }
        }
    }   
}

pub struct BlocActionPlugin;

impl Plugin for BlocActionPlugin {
    fn build(&self, app: &mut App) {
        app
			.add_systems(Update, target_bloc)
			.add_systems(Update, bloc_outline)
            .add_systems(Update, break_bloc.run_if(in_state(GameState::Game)))
            .add_systems(Update, place_bloc.run_if(in_state(GameState::Game)))
			;
    }
}