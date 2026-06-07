mod block_entities;
mod chunk;
mod load_area;
mod utils;
mod voxel_world;
use bevy::prelude::*;
pub use block_entities::BlockEntities;
pub use block_entities::unload_block_entities;
pub use chunk::*;
pub use load_area::*;
use rand_chacha::ChaCha8Rng;
pub use rb_pos::*;
pub use voxel_world::*;

pub const RENDER_DISTANCE: i32 = 32;

#[derive(Component, Default)]
pub struct PlayerCol(pub ChunkPos2d);

#[derive(Message)]
pub struct ColUnloadEvent(pub ChunkPos2d);

#[derive(Resource)]
pub struct WorldRng {
    pub seed: u64,
    pub rng: ChaCha8Rng,
}
