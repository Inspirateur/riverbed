use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use crate::GameState;
use crate::agents::{AABB, Dir};
use leafwing_input_manager::prelude::*;

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


pub struct Camera3dPlugin;

impl Plugin for Camera3dPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, apply_fps_cam)
            .add_systems(Update, pan_camera.run_if(in_state(GameState::Game)))
        ;
    }
}