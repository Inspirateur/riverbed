use itertools::Itertools;
use packed_uints::PackedUints;
use super::{Bloc, pos::{ChunkedPos, ColedPos}, utils::Palette, CHUNK_S1, CHUNK_S3, CHUNK_SHAPE};

#[derive(Debug)]
pub struct Chunk {
    data: PackedUints,
    palette: Palette<Bloc>,
}

impl Chunk {
    pub fn get(&self, (x, y, z): ChunkedPos) -> &Bloc {
        &self.palette[self.data.get(CHUNK_SHAPE.linearize(x, y, z))]
    }

    pub fn set(&mut self, (x, y, z): ChunkedPos, bloc: Bloc) {
        let idx = CHUNK_SHAPE.linearize(x, y, z);
        self.data.set(idx, self.palette.index(bloc));
    }

    pub fn set_yrange(&mut self, (x, top, z): ChunkedPos, height: usize, bloc: Bloc) {
        let value = self.palette.index(bloc);
        let top = CHUNK_SHAPE.linearize(x, top, z);
        let bottom = top - height + 1;
        self.data.set_range(bottom, top + 1, value);
    }

    /// Used for efficient construction of mesh data
    pub fn copy_column(&self, buffer: &mut [Bloc], (x, z): ColedPos, lod: usize) {
        let start = CHUNK_SHAPE.linearize(x, 0, z);
        let mut i = 0;
        for idx in (start..(start+CHUNK_S1)).step_by(lod) {
            buffer[i] = self.palette[self.data.get(idx)];
            i += 1;
        }
    }

    pub fn top(&self, (x, z): ColedPos) -> (&Bloc, usize) {
        for y in (0..CHUNK_S1).rev() {
            let b_idx = self.data.get(CHUNK_SHAPE.linearize(x, y, z));
            if b_idx > 0 {
                return (&self.palette[b_idx], y);
            }
        }
        (&self.palette[0], 0)
    }

    pub fn set_if_empty(&mut self, (x, y, z): ChunkedPos, bloc: Bloc) -> bool {
        let idx = CHUNK_SHAPE.linearize(x, y, z);
        if self.palette[self.data.get(idx)] != Bloc::default() {
            return false;
        }
        self.data.set(idx, self.palette.index(bloc));
        true
    }
}

impl From<&[Bloc]> for Chunk {
    fn from(values: &[Bloc]) -> Self {
        let mut palette = Palette::new();
        palette.index(Bloc::default());
        let values = values.iter().map(|v| palette.index(v.clone())).collect_vec();
        let data = PackedUints::from(values.as_slice());
        Chunk {data, palette}
    }
}

impl Chunk {
    pub fn new() -> Self {
        let mut palette = Palette::new();
        palette.index(Bloc::default()); 
        Chunk {
            data: PackedUints::new(CHUNK_S3),
            palette: palette, 
        }
    }
}