use itertools::Itertools;
use packed_uints::PackedUints;
use crate::blocks::Block;
use super::{pos::{ChunkedPos, ColedPos}, utils::Palette, CHUNKP_S1, CHUNKP_S2, CHUNKP_S3, CHUNK_S1};

#[derive(Debug)]
pub struct Chunk {
    pub data: PackedUints,
    pub palette: Palette<Block>,
}

pub fn linearize(x: usize, y: usize, z: usize) -> usize {
    z + x * CHUNKP_S1 + y * CHUNKP_S2
}

pub fn pad_linearize(x: usize, y: usize, z: usize) -> usize {
    z + 1 + (x+1) * CHUNKP_S1 + (y+1) * CHUNKP_S2
}

impl Chunk {
    pub fn get(&self, (x, y, z): ChunkedPos) -> &Block {
        &self.palette[self.data.get(pad_linearize(x, y, z))]
    }

    pub fn set(&mut self, (x, y, z): ChunkedPos, block: Block) {
        let idx = pad_linearize(x, y, z);
        self.data.set(idx, self.palette.index(block));
    }

    pub fn set_yrange(&mut self, (x, top, z): ChunkedPos, height: usize, block: Block) {
        let value = self.palette.index(block);
        // Note: we do end+1 because set_range(_step) is not inclusive
        self.data.set_range_step(
            pad_linearize(x, top - height, z), 
            pad_linearize(x, top, z)+1, 
            CHUNKP_S2,
            value
        );
    }

    // Used for efficient construction of mesh data
    pub fn copy_column(&self, buffer: &mut [Block], (x, z): ColedPos, lod: usize) {
        let start = pad_linearize(x, 0, z);
        let mut i = 0;
        for idx in (start..(start+CHUNK_S1)).step_by(lod) {
            buffer[i] = self.palette[self.data.get(idx)];
            i += 1;
        }
    }

    pub fn top(&self, (x, z): ColedPos) -> (&Block, usize) {
        for y in (0..CHUNK_S1).rev() {
            let b_idx = self.data.get(pad_linearize(x, y, z));
            if b_idx > 0 {
                return (&self.palette[b_idx], y);
            }
        }
        (&self.palette[0], 0)
    }

    pub fn set_if_empty(&mut self, (x, y, z): ChunkedPos, block: Block) -> bool {
        let idx = pad_linearize(x, y, z);
        if self.palette[self.data.get(idx)] != Block::Air {
            return false;
        }
        self.data.set(idx, self.palette.index(block));
        true
    }
}

impl From<&[Block]> for Chunk {
    fn from(values: &[Block]) -> Self {
        let mut palette = Palette::new();
        palette.index(Block::Air);
        let values = values.iter().map(|v| palette.index(v.clone())).collect_vec();
        let data = PackedUints::from(values.as_slice());
        Chunk {data, palette}
    }
}

impl Chunk {
    pub fn new() -> Self {
        let mut palette = Palette::new();
        palette.index(Block::Air); 
        Chunk {
            data: PackedUints::new(CHUNKP_S3),
            palette: palette, 
        }
    }
}