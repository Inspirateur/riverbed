use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use shared::{block::{Block, Face}, world::{chunk::Chunk, pos::pos3d::ChunkPos, utils::{Palette, SerdablePackedUints}}};

#[derive(Debug)]
pub struct ClientChunk(Chunk);

impl ClientChunk {
    pub fn new(data: SerdablePackedUints, palette: Palette<Block>) -> Self {
        Self(Chunk { data, palette })
    }

    /// Create a ClientChunk from a shared Chunk (used when receiving from network)
    pub fn from_chunk(chunk: Chunk) -> Self {
        Self(chunk)
    }

    pub fn get(&self, pos: (usize, usize, usize)) -> &Block {
        self.0.get(pos)
    }

    pub fn set(&mut self, pos: (usize, usize, usize), block: Block) {
        self.0.set(pos, block);
    }

    pub fn set_unpadded(&mut self, pos: (usize, usize, usize), block: Block) {
        self.0.set_unpadded(pos, block);
    }

    pub fn set_yrange(&mut self, pos: (usize, usize, usize), height: usize, block: Block) {
        self.0.set_yrange(pos, height, block);
    }

    pub fn top(&self, pos: (usize, usize)) -> (&Block, usize) {
        self.0.top(pos)
    }

    pub fn set_if_empty(&mut self, pos: (usize, usize, usize), block: Block) -> bool {
        self.0.set_if_empty(pos, block)
    }

    pub fn copy_side_from(&mut self, other: &ClientChunk, face: Face) {
        self.0.copy_side_from(&other.0, face);
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
        Self::from_chunk(chunk)
    }
}


#[derive(Resource)]
pub struct ClientWorld {
    pub name: String,
    /// Data the client needs to render
    pub chunks: HashMap<ChunkPos, Arc<ClientChunk>>,
}

impl Default for ClientWorld {
    fn default() -> Self {
        Self {
            name: String::new(),
            chunks: HashMap::new(),
        }
    }
}
