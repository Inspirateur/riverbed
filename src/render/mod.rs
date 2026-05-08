mod chunk_culling;
mod camera;
mod mesh_draw;
mod mesh_thread;
mod mesh_utils;
mod mesh_logic;
mod texture_load;
mod texture_array;
mod voxel_grid_mesh_draw;
mod voxel_grid_mesh_thread;
mod sky;
mod effects;
use bevy::prelude::Plugin;
pub use texture_load::*;
pub use texture_array::TextureMap;
pub use camera::{FpsCam, CameraSpawn};
pub use mesh_draw::ChunkColliderEntities;
pub use mesh_thread::{MeshOrderReceiver, MeshOrderSender};
pub use voxel_grid_mesh_draw::{GridChildEntities, spawn_voxel_grid};

pub struct Render;

impl Plugin for Render {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.add_plugins(mesh_draw::Draw3d)
			.add_plugins(voxel_grid_mesh_draw::VoxelGridPlugin)
			.add_plugins(sky::SkyPlugin)
			.add_plugins(camera::Camera3dPlugin)
			.add_plugins(effects::EffectsPlugin)
			;
    }
}