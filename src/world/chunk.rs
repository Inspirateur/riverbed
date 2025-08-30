use itertools::Itertools;
use packed_uints::PackedUints;
use crate::{block::Face, world::CHUNK_S1I, Block};
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

    pub fn set_unpadded(&mut self, (x, y, z): ChunkedPos, block: Block) {
        let idx = linearize(x, y, z);
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

    pub fn copy_side_from(&mut self, other: &Chunk, face: Face) {
        // TODO: doesn't work, row step or col step are probably wrong
        let row_step = match face {
            Face::Left | Face::Right => CHUNKP_S1,
            Face::Down | Face::Up => CHUNKP_S2,
            Face::Back | Face::Front => 1,
        };
        let col_step = match face {
            Face::Left | Face::Right => CHUNKP_S2,
            Face::Down | Face::Up => 1,
            Face::Back | Face::Front => CHUNKP_S1,
        } - row_step * CHUNK_S1;
        let [x, y, z] = face.n();
        let mut self_i = linearize(
            (x * (CHUNK_S1I + 1)).max(0) as usize,
            (y * (CHUNK_S1I + 1)).max(0) as usize, 
            (z * (CHUNK_S1I + 1)).max(0) as usize,
        );
        let [x, y, z] = face.opposite().n();
        let mut other_i= linearize(
            (x * CHUNK_S1I).max(1) as usize,
            (y * CHUNK_S1I).max(1) as usize, 
            (z * CHUNK_S1I).max(1) as usize,
        );
        for _ in 0..CHUNK_S1 {
            for _ in 0..CHUNK_S1 {
                self.data.set(self_i, other.data.get(other_i));
                self_i += row_step;
                other_i += row_step;
            }
            self_i += col_step;
            other_i += col_step;
        }
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