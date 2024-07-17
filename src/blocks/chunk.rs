use itertools::Itertools;
use packed_uints::PackedUints;
use super::{Block, pos::{ChunkedPos, ColedPos}, utils::Palette, CHUNK_S1, CHUNK_S3, CHUNK_SHAPE};

#[derive(Debug)]
pub struct Chunk {
    data: PackedUints,
    palette: Palette<Block>,
}

impl Chunk {
    pub fn get(&self, (x, y, z): ChunkedPos) -> &Block {
        &self.palette[self.data.get(CHUNK_SHAPE.linearize(x, y, z))]
    }

    pub fn set(&mut self, (x, y, z): ChunkedPos, block: Block) {
        let idx = CHUNK_SHAPE.linearize(x, y, z);
        self.data.set(idx, self.palette.index(block));
    }

    pub fn set_yrange(&mut self, (x, top, z): ChunkedPos, height: usize, block: Block) {
        let value = self.palette.index(block);
        let top = CHUNK_SHAPE.linearize(x, top, z) + 1;
        let bottom = top - height;
        self.data.set_range(bottom, top, value);
    }

    // Used for efficient construction of mesh data
    pub fn copy_column(&self, buffer: &mut [Block], (x, z): ColedPos, lod: usize) {
        let start = CHUNK_SHAPE.linearize(x, 0, z);
        let mut i = 0;
        for idx in (start..(start+CHUNK_S1)).step_by(lod) {
            buffer[i] = self.palette[self.data.get(idx)];
            i += 1;
        }
    }

    pub fn top(&self, (x, z): ColedPos) -> (&Block, usize) {
        for y in (0..CHUNK_S1).rev() {
            let b_idx = self.data.get(CHUNK_SHAPE.linearize(x, y, z));
            if b_idx > 0 {
                return (&self.palette[b_idx], y);
            }
        }
        (&self.palette[0], 0)
    }

    pub fn set_if_empty(&mut self, (x, y, z): ChunkedPos, block: Block) -> bool {
        let idx = CHUNK_SHAPE.linearize(x, y, z);
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
            data: PackedUints::new(CHUNK_S3),
            palette: palette, 
        }
    }
}