use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use crate::GameState;
use crate::agents::{PlayerControlled, PlayerSpawn, AABB};
use leafwing_input_manager::prelude::*;

const CAMERA_PAN_RATE: f32 = 0.06;

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
pub enum CameraMovement {
    Pan,
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CameraSpawn;

pub fn cam_setup(mut commands: Commands, mut windows: Query<&mut Window>, player_query: Query<(Entity, &AABB), With<PlayerControlled>>) {
    let input_map = InputMap::new([
        // This will capture the total continuous value, for direct use.
        // Note that you can also use discrete gesture-like motion,
        // via the `MouseMotionDirection` enum.
        (CameraMovement::Pan, DualAxis::mouse_motion()),
    ]);
    let (player, aabb) = player_query.get_single().unwrap();
    let cam = commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(aabb.0.x/2., 2., aabb.0.z/2.)
            .looking_at(Vec3 {x: 0., y: 0., z: 1.}, Vec3::Y),
        // we put the far clipping plane very far because we don't want any chunks behind it anyway
        projection: Projection::Perspective(PerspectiveProjection { 
            far: 10000.,
            fov: FRAC_PI_2 * 9./16., 
            ..Default::default() 
        }),
        ..Default::default()
    },
    FogSettings {
        color: Color::rgba(0.70, 0.85, 0.95, 1.0),
        falloff: FogFalloff::Linear {
            start: 100.0,
            end: 5000.0,
        },
        ..default()
    }))
    .insert(InputManagerBundle::<CameraMovement> {
        input_map,
        ..default()
    }).insert(FpsCam::default()).id();
    commands.entity(player).add_child(cam);
    let mut window = windows.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

pub fn pan_camera(mut query: Query<(&ActionState<CameraMovement>, &mut FpsCam)>, time: Res<Time>) {
    let (action_state, mut fpscam) = query.single_mut();
    let camera_pan_vector = action_state.axis_pair(&CameraMovement::Pan).unwrap();
    let c = time.delta_seconds() * CAMERA_PAN_RATE;
    fpscam.yaw -= c*camera_pan_vector.x();
    fpscam.pitch -= c*camera_pan_vector.y();
    fpscam.pitch = fpscam.pitch.clamp(-1.5, 1.5);
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
            .add_systems(Startup, (cam_setup, apply_deferred).chain().in_set(CameraSpawn).after(PlayerSpawn))
            .add_systems(Update, apply_fps_cam)
            .add_systems(Update, pan_camera.run_if(in_state(GameState::Game)))
        ;
    }
}