use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CameraSpawn;
