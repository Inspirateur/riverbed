use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use bevy::{pbr::VolumetricFog, prelude::*};
use bevy::window::CursorGrabMode;
use crate::agents::Velocity;
use crate::{agents::{PlayerControlled, PlayerSpawn, AABB}, ui::CursorGrabbed};
use leafwing_input_manager::prelude::*;

const CAMERA_PAN_RATE: f32 = 0.001;

pub struct Camera3dPlugin;

impl Plugin for Camera3dPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_systems(Startup, (cam_setup, ApplyDeferred).chain().in_set(CameraSpawn).after(PlayerSpawn))
            .add_systems(Update, apply_fps_cam)
            .add_systems(Update, adaptative_fov)
            .add_systems(Update, pan_camera.run_if(in_state(CursorGrabbed)))
        ;
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
pub enum CameraMovement {
    Pan,
}

impl Actionlike for CameraMovement {
    fn input_control_kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CameraSpawn;

pub fn cam_setup(mut commands: Commands, mut windows: Query<&mut Window>, player_query: Query<(Entity, &AABB), With<PlayerControlled>>) {
    let input_map = InputMap::default()
        // This will capture the total continuous value, for direct use.
        // Note that you can also use discrete gesture-like motion,
        // via the `MouseMotionDirection` enum.
        .with_dual_axis(CameraMovement::Pan, MouseMove::default());
    let (player, aabb) = player_query.single().unwrap();
    let cam = commands
        .spawn((
                Camera3d::default(),
                Transform::from_xyz(aabb.0.x/2., aabb.0.y-0.05, aabb.0.z/2.)
                    .looking_at(Vec3 {x: 0., y: 0., z: 1.}, Vec3::Y),
                Projection::Perspective(PerspectiveProjection { 
                    far: 10000.,
                    fov: FRAC_PI_2 * 9./16., 
                    ..Default::default() 
                }),
                DistanceFog {
                    color: Color::linear_rgba(0.70, 0.85, 0.95, 1.0),
                    falloff: FogFalloff::Linear {
                        start: 100.0,
                        end: 10000.0,
                    },
                    ..default()
                },
                VolumetricFog::default()
            )
        )
        // TODO: We would like SSAO very much but it doesn't like that Mesh data is compressed
        //.insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(input_map)
        .insert(FpsCam::default())
        .id();
    commands.entity(player).add_child(cam);
    let mut window = windows.single_mut().unwrap();
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

// This system adapts the camera's field of view based on the player's speed
pub fn adaptative_fov(
    cam_query: Single<(&Transform, &mut Projection)>,
    player_query: Single<&Velocity, With<PlayerControlled>>,
    time: Res<Time>,
) {
    let velocity = player_query.into_inner();
    let (transform, mut perspective) = cam_query.into_inner();
    // Adjust the FOV based on the player's speed
    if let Projection::Perspective(projection) = &mut *perspective {
        let speed = transform.rotation.mul_vec3(-Vec3::Z).dot(velocity.0);
        let target_fov = FRAC_PI_4 * (speed / 10.0).clamp(1.0, 2.0);
        projection.fov = projection.fov.lerp(target_fov, time.delta_secs() * 4.0);
    }
}

pub fn pan_camera(mut query: Query<(&ActionState<CameraMovement>, &mut FpsCam)>) {
    let (action_state, mut fpscam) = query.single_mut().unwrap();
    let camera_pan_vector = action_state.axis_pair(&CameraMovement::Pan);
    fpscam.yaw -= CAMERA_PAN_RATE*camera_pan_vector.x;
    fpscam.pitch -= CAMERA_PAN_RATE*camera_pan_vector.y;
    fpscam.pitch = fpscam.pitch.clamp(-1.5, 1.5);
}

pub fn apply_fps_cam(mut query: Query<(&mut Transform, &FpsCam)>) {
    let (mut transform, fpscam) = query.single_mut().unwrap();
    transform.rotation = Quat::from_axis_angle(Vec3::Y, fpscam.yaw) * Quat::from_axis_angle(Vec3::X, fpscam.pitch);
}
