use shared::{
    block::Block,
    world::{
        chunk::Chunk,
        utils::{Palette, SerdablePackedUints},
    },
};

#[derive(Debug)]
pub struct ClientChunk(Chunk);

impl ClientChunk {
    pub fn from_network_chunk(chunk: Chunk) -> Self {
        Self(chunk)
    }

    pub fn get(&self, pos: (usize, usize, usize)) -> &Block {
        self.0.get(pos)
    }

    pub fn set(&mut self, pos: (usize, usize, usize), block: Block) {
        self.0.set(pos, block);
    }

    pub fn data(&self) -> &SerdablePackedUints {
        &self.0.data
    }

    pub fn palette(&self) -> &Palette<Block> {
        &self.0.palette
    }
}

impl From<Chunk> for ClientChunk {
    fn from(chunk: Chunk) -> Self {
        Self::from_network_chunk(chunk)
    }
}
