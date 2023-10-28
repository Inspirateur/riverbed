use std::iter::zip;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use crate::blocs::{Pos, Blocs};
use crate::agents::{AABB, TargetBloc, Dir};
use leafwing_input_manager::prelude::*;

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
const CAMERA_PAN_RATE: f32 = 0.1;

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect)]
pub enum CameraMovement {
    Pan,
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 150., 10.)
            .looking_at(Vec3 {x: 0., y: 150., z: 0.}, Vec3::Y),
        ..Default::default()
    })
    .insert(InputManagerBundle::<CameraMovement> {
        input_map: InputMap::default()
            // This will capture the total continuous value, for direct use.
            // Note that you can also use discrete gesture-like motion, via the `MouseMotionDirection` enum.
            .insert(DualAxis::mouse_motion(), CameraMovement::Pan)
            .build(),
        ..default()
    }).insert(FpsCam::default());
    let mut window = windows.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

pub fn translate_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<(&Pos, &AABB), (With<ActionState<Dir>>, Changed<Pos>)>
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok((player_pos, aabb)) = player_query.get_single() {
            cam_pos.translation.x = player_pos.x + aabb.0.x/2.;
            cam_pos.translation.y = player_pos.y + aabb.0.y;
            cam_pos.translation.z = player_pos.z + aabb.0.z/2.;
        }
    }
}

pub fn pan_camera(mut query: Query<(&ActionState<CameraMovement>, &mut FpsCam)>, time: Res<Time>) {
    let (action_state, mut fpscam) = query.single_mut();
    let camera_pan_vector = action_state.axis_pair(CameraMovement::Pan).unwrap();
    let c = time.delta_seconds() * CAMERA_PAN_RATE;
    fpscam.yaw -= c*camera_pan_vector.x();
    fpscam.pitch -= c*camera_pan_vector.y();
    fpscam.pitch = fpscam.pitch.clamp(-1.4, 1.4);
}

pub fn apply_fps_cam(mut query: Query<(&mut Transform, &FpsCam)>) {
    let (mut transform, fpscam) = query.single_mut();
    transform.rotation = Quat::from_axis_angle(Vec3::Y, fpscam.yaw) * Quat::from_axis_angle(Vec3::X, fpscam.pitch);
}

pub fn target_bloc(
    mut player: Query<(&mut TargetBloc, &Pos<f32>), With<ActionState<Dir>>>, 
    player_cam: Query<&Transform, With<FpsCam>>,
    world: Res<Blocs>
) {
    let (mut target_bloc, player_pos) = player.single_mut();
    let transform = player_cam.single();
    target_bloc.0 = world.raycast(
        player_pos.realm, 
        transform.translation, 
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
