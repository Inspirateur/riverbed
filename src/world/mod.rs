mod load_area;
mod block_entities;
mod voxel_world;
mod terrain_thread;
mod realm;
mod chunk;
mod pos;
mod utils;
pub use realm::*;
pub use voxel_world::*;
pub use chunk::*;
pub use pos::*;
pub use block_entities::BlockEntities;
use bevy::prelude::*;
use crate::world::{block_entities::unload_block_entities, terrain_thread::{assign_player_col, on_unload_col, send_player_pos_update, setup_load_thread}};
pub const CHUNK_S1: usize = 62;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNKP_S1: usize = CHUNK_S1 + 2;
pub const CHUNKP_S2: usize = CHUNKP_S1.pow(2);
pub const CHUNKP_S3: usize = CHUNKP_S1.pow(3);
pub const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub const MAX_HEIGHT: usize = 496;
pub const MAX_GEN_HEIGHT: usize = 400;
pub const WATER_H: i32 = 61;
pub const Y_CHUNKS: usize = MAX_HEIGHT/CHUNK_S1;

#[derive(Component, Default)]
pub struct PlayerCol(pub ColPos);

#[derive(Event)]
pub struct ColUnloadEvent(pub ColPos);

#[derive(Event)]
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