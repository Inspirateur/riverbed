use bevy::prelude::*;
use rb_camera::{CameraSpawn, FpsCam};

pub struct AutoCameraPlugin;

impl Plugin for AutoCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_auto_camera.in_set(CameraSpawn));
    }
}

fn setup_auto_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        FpsCam::default(),
        Transform::from_xyz(500., 180., 300.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
