mod terrain_thread;
mod block_entities;
pub mod voxel_world;

use bevy::prelude::*;
use shared::world::{block_entities::BlockEntities, pos::{pos2d::ColPos, pos3d::ChunkPos}};
use crate::world::{block_entities::unload_block_entities, terrain_thread::{assign_player_col, on_unload_col, send_player_pos_update, setup_load_thread}};

#[derive(Message)]
pub struct ColUnloadEvent(pub ColPos);

#[derive(Message)]
pub struct ChunkChanged(pub ChunkPos);

pub struct TerrainLoadPlugin;

impl Plugin for TerrainLoadPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.add_event::<ColUnloadEvent>()
			.insert_resource(BlockEntities::default())
			.add_systems(Startup, setup_load_thread)
			.add_systems(Update, send_player_pos_update)
			.add_systems(Update, assign_player_col)
			.add_systems(Update, on_unload_col)
			.add_systems(Update, unload_block_entities)
		;
	}
}