mod chunk_culling;
mod camera;
mod draw_chunks;
mod mesh_utils;
mod mesh_chunks;
mod texture_load;
mod texture_array;
mod sky;
mod shared_load_area;
mod effects;
use bevy::prelude::Plugin;
pub use texture_load::*;
pub use camera::{FpsCam, CameraSpawn};

pub struct Render;

impl Plugin for Render {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.add_plugins(draw_chunks::Draw3d)
			.add_plugins(sky::SkyPlugin)
			.add_plugins(camera::Camera3dPlugin)
			.add_plugins(effects::EffectsPlugin)
			;
    }
}