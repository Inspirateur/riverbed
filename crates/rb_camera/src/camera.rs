use bevy::prelude::*;
use rb_block::Block;
use rb_physics::Velocity;
use rb_world::{Realm, VoxelWorld};
use std::f32::consts::FRAC_PI_4;

const AIR_FALOFF: FogFalloff = FogFalloff::Atmospheric {
    extinction: Vec3::new(0.0003, 0.0002, 0.0001),
    inscattering: Vec3::new(0.0001, 0.0002, 0.0003),
};
const WATER_FALOFF: FogFalloff = FogFalloff::Atmospheric {
    extinction: Vec3::new(0.003, 0.002, 0.001),
    inscattering: Vec3::new(0.0, 0.0, 0.0),
};
const AIR_COLOR: Color = Color::linear_rgba(0.70, 0.85, 0.95, 1.0);
const WATER_COLOR: Color = Color::linear_rgba(0.05, 0.1, 0.2, 1.0);

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct CameraSpawn;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

pub struct Camera3dPlugin;

impl Plugin for Camera3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_fps_cam)
            .add_systems(Update, adaptative_fov)
            .add_systems(Update, water_fog);
    }
}

fn adaptative_fov(
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

fn apply_fps_cam(mut query: Query<(&mut Transform, &FpsCam)>) {
    let (mut transform, fpscam) = query.single_mut().unwrap();
    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, fpscam.yaw) * Quat::from_axis_angle(Vec3::X, fpscam.pitch);
}

fn water_fog(
    cam: Single<(&mut DistanceFog, &GlobalTransform)>,
    player: Single<&Realm, With<PlayerControlled>>,
    voxel_world: Res<VoxelWorld>,
) {
    let (mut fog, transform) = cam.into_inner();
    let &realm = player.into_inner();
    let block_pos = (transform.translation(), realm).into();
    if voxel_world.get_block(block_pos) == Block::SeaBlock {
        fog.color = WATER_COLOR;
        fog.falloff = WATER_FALOFF;
    } else {
        fog.color = AIR_COLOR;
        fog.falloff = AIR_FALOFF;
    }
}
