mod block_entities;
mod chunk;
mod load_area;
mod utils;
mod voxel_world;
pub use rb_pos::*;
pub use block_entities::BlockEntities;
pub use block_entities::unload_block_entities;
pub use chunk::*;
pub use voxel_world::*;
pub use load_area::*;
use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;

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
