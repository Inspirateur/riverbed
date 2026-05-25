use bevy::prelude::*;
use rb_camera::CameraSpawn;

pub struct AutoCameraPlugin;

impl Plugin for AutoCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_auto_camera.in_set(CameraSpawn));
    }
}

fn setup_auto_camera() {}
