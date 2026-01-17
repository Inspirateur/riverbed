use shared::world::{chunk::Chunk, pos::ChunkPos};

pub struct ClientWorld {
    pub name: String,
    pub chunks: HashMap<ChunkPos, Arc<Chunk>>,
}
