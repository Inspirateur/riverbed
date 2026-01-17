mod camera;
mod chunk_culling;
mod effects;
mod mesh_draw;
mod mesh_logic;
mod mesh_thread;
mod sky;
mod texture_array;
mod texture_load;
use bevy::prelude::Plugin;
pub use camera::{CameraSpawn, FpsCam};
pub use mesh_thread::{MeshOrderReceiver, MeshOrderSender};
pub use texture_load::*;

pub struct Render;

impl Plugin for Render {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(mesh_draw::Draw3d)
            .add_plugins(sky::SkyPlugin)
            .add_plugins(camera::Camera3dPlugin)
            .add_plugins(effects::EffectsPlugin);
    }
}
