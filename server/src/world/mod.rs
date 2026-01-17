mod chunk;
mod terrain_thread;
mod serdable_packed_uints;
mod block_entities;
mod voxel_world;

use bevy::prelude::*;
use shared::world::{block_entities::BlockEntities, pos::{ChunkPos, ColPos}};
use crate::world::{block_entities::unload_block_entities, terrain_thread::{assign_player_col, on_unload_col, send_player_pos_update, setup_load_thread}};

pub use shared::world::{
	CHUNK_S1,
	CHUNK_S2,
	CHUNKP_S1,
	CHUNKP_S2,
	CHUNKP_S3,
	CHUNK_S1I,
	MAX_HEIGHT,
	MAX_GEN_HEIGHT,
	WATER_H,
	Y_CHUNKS,
};


#[derive(Component, Default)]
pub struct PlayerCol(pub ColPos);

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