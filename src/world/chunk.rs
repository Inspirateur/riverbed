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
        // TODO: there's still a problem somewhere. Some faces are invisible when they should be visible
        let row_step = match face {
            Face::Left | Face::Right => 1,
            Face::Down | Face::Up => 1,
            Face::Back | Face::Front => CHUNKP_S1,
        };
        let col_step = match face {
            Face::Left | Face::Right => CHUNKP_S2 - CHUNK_S1,
            Face::Down | Face::Up => CHUNKP_S1 - CHUNK_S1,
            Face::Back | Face::Front => CHUNKP_S1 + CHUNKP_S1,
        };
        let [nx, ny, nz] = face.n();
        let mut self_i = linearize(
            ((nx * CHUNK_S1I).max(1) + nx) as usize,
            ((ny * CHUNK_S1I).max(1) + ny) as usize, 
            ((nz * CHUNK_S1I).max(1) + nz) as usize,
        );
        let [nx, ny, nz] = face.opposite().n();
        let mut other_i= linearize(
            (nx * CHUNK_S1I).max(1) as usize,
            (ny * CHUNK_S1I).max(1) as usize, 
            (nz * CHUNK_S1I).max(1) as usize,
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

#[cfg(test)]
mod tests {
    use crate::{block::Face, world::{CHUNK_S1, CHUNK_S1I}};

    fn print_chunk_face_indices(face: Face) {
        let [nx, ny, nz] = face.n();
        let x = ((nx * CHUNK_S1I).max(1) + nx) as usize;
        let y = ((ny * CHUNK_S1I).max(1) + ny) as usize;
        let z = ((nz * CHUNK_S1I).max(1) + nz) as usize;
        eprintln!("{x}, {y}, {z}");
        let [tx, ty, tz] = face.t();
        for dy in 0..(CHUNK_S1*ty).max(1) {
            for dx in 0..(CHUNK_S1*tx).max(1) {
                for dz in 0..(CHUNK_S1*tz).max(1) {
                    let idx = super::linearize(x + dx, y + dy, z + dz);
                    eprint!("{}, ", idx);
                }
            }
        }
    }

    #[test]
    fn test_face_idx() {
        print_chunk_face_indices(Face::Front);
    }
}