mod chunk_culling;
mod mesh_draw;
mod mesh_logic;
mod mesh_thread;
mod mesh_utils;
mod sky;
mod texture_array;
mod texture_load;
use bevy::prelude::Plugin;
pub use mesh_thread::{MeshOrderReceiver, MeshOrderSender};
pub use texture_load::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(mesh_draw::Draw3d)
            .add_plugins(sky::SkyPlugin);
    }
}
