mod camera;
mod draw3d;
mod render3d;
mod texture_array;
mod block_action;
mod sky;
mod shared_load_area;
use bevy::prelude::Plugin;

pub struct Game3d;

impl Plugin for Game3d {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.add_plugins(draw3d::Draw3d)
			.add_plugins(sky::SkyPlugin)
			.add_plugins(camera::Camera3dPlugin)
			.add_plugins(block_action::BlockActionPlugin)
			;
    }
}