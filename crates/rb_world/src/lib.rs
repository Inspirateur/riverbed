mod block_entities;
mod chunk;
mod column;
mod load_area;
mod structure_trait;
mod utils;
mod voxel_world;
use bevy::prelude::*;
pub use block_entities::BlockEntities;
pub use block_entities::unload_block_entities;
pub use chunk::*;
pub use column::*;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
pub use load_area::*;
use rand_chacha::ChaCha8Rng;
pub use rb_pos::*;
pub use structure_trait::StructureTrait;
pub use voxel_world::*;

pub const RENDER_DISTANCE: i32 = 32;
pub const MAX_HEIGHT: usize = 496;
pub const MAX_GEN_HEIGHT: usize = 400;
pub const WATER_H: i32 = 61;
pub const Y_CHUNKS: usize = MAX_HEIGHT / CHUNK_S1;

#[derive(Component, Default)]
pub struct PlayerCol(pub ChunkPos2d);

#[derive(Resource)]
pub struct ChunkEventSender(pub Sender<(ChunkEvent, ChunkPos)>);

#[derive(Resource)]
pub struct ChunkEventReceiver(pub Receiver<(ChunkEvent, ChunkPos)>);

#[derive(Message)]
pub struct ColUnloadEvent(pub ChunkPos2d);

#[derive(Resource)]
pub struct WorldRng {
    pub seed: u64,
    pub rng: ChaCha8Rng,
}

pub fn chunks_in_col(col_pos: &ChunkPos2d) -> [ChunkPos; Y_CHUNKS] {
    std::array::from_fn(|y| ChunkPos {
        x: col_pos.x,
        y: y as i32,
        z: col_pos.z,
        realm: col_pos.realm,
    })
}
