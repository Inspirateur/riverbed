mod block_entities;
mod chunk;
mod load_area;
mod pos;
mod realm;
mod terrain_thread;
mod utils;
mod voxel_world;
use crate::world::{
    block_entities::unload_block_entities,
    terrain_thread::{assign_player_col, on_unload_col, send_player_pos_update, setup_load_thread},
};
use bevy::prelude::*;
pub use block_entities::BlockEntities;
pub use chunk::*;
pub use pos::*;
pub use realm::*;
pub use voxel_world::*;
pub const CHUNK_S1: usize = 62;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNKP_S1: usize = CHUNK_S1 + 2;
pub const CHUNKP_S2: usize = CHUNKP_S1.pow(2);
pub const CHUNKP_S3: usize = CHUNKP_S1.pow(3);
pub const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub const MAX_HEIGHT: usize = 496;
pub const MAX_GEN_HEIGHT: usize = 400;
pub const WATER_H: i32 = 61;
pub const Y_CHUNKS: usize = MAX_HEIGHT / CHUNK_S1;

#[derive(Component, Default)]
pub struct PlayerCol(pub ColPos);

#[derive(Message)]
pub struct ColUnloadEvent(pub ColPos);

pub struct TerrainLoadPlugin;

impl Plugin for TerrainLoadPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_message::<ColUnloadEvent>()
            .insert_resource(BlockEntities::default())
            .add_systems(Startup, setup_load_thread)
            .add_systems(Update, send_player_pos_update)
            .add_systems(Update, assign_player_col)
            .add_systems(Update, on_unload_col)
            .add_systems(Update, unload_block_entities);
    }
}
