mod chunk_culling;
mod camera;
mod draw3d;
mod render3d;
mod texture_load;
mod texture_array;
mod sky;
mod shared_load_area;
use bevy::prelude::Plugin;
pub use texture_load::*;
pub use camera::FpsCam;

pub struct Render;

impl Plugin for Render {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.add_plugins(draw3d::Draw3d)
			.add_plugins(sky::SkyPlugin)
			.add_plugins(camera::Camera3dPlugin)
			;
    }
}