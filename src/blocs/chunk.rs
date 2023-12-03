use itertools::Itertools;
use packed_uints::PackedUints;
use super::{Bloc, pos::{ChunkedPos, ColedPos}, utils::Palette};
pub const CHUNK_S1: usize = 32;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNK_S3: usize = CHUNK_S1.pow(3);

#[derive(Debug)]
pub struct Chunk {
    data: PackedUints,
    palette: Palette<Bloc>,
    size: usize,
    size2: usize
}

impl Chunk {
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        // arranged by columns for efficiency of vertical operations
        y + x * self.size + z * self.size2
    }

    pub fn get(&self, (x, y, z): ChunkedPos) -> &Bloc {
        &self.palette[self.data.get(self.index(x, y, z))]
    }

    pub fn set(&mut self, (x, y, z): ChunkedPos, bloc: Bloc) {
        let idx = self.index(x, y, z);
        self.data.set(idx, self.palette.index(bloc));
    }

    pub fn set_yrange(&mut self, (x, top, z): ChunkedPos, height: usize, bloc: Bloc) {
        let value = self.palette.index(bloc);
        let top = self.index(x, top, z);
        for idx in (top-height+1)..=top {
            self.data.set(idx, value);
        }
    }

    pub fn top(&self, (x, z): ColedPos) -> (&Bloc, usize) {
        for y in (0..self.size).rev() {
            let b_idx = self.data.get(self.index(x, y, z));
            if b_idx > 0 {
                return (&self.palette[b_idx], y);
            }
        }
        (&self.palette[0], 0)
    }

    pub fn set_if_empty(&mut self, (x, y, z): ChunkedPos, bloc: Bloc) -> bool {
        let idx = self.index(x, y, z);
        if self.palette[self.data.get(idx)] != Bloc::default() {
            return false;
        }
        self.data.set(idx, self.palette.index(bloc));
        true
    }
}

impl From<&[Bloc]> for Chunk {
    fn from(values: &[Bloc]) -> Self {
        let size = (values.len() as f64).cbrt() as usize;
        let mut palette = Palette::new();
        palette.index(Bloc::default());
        let values = values.iter().map(|v| palette.index(v.clone())).collect_vec();
        let data = PackedUints::from(values.as_slice());
        Chunk {
            data, palette, 
            size, size2: size*size
        }
    }
}

impl Chunk {
    pub fn new(size: usize) -> Self {
        let mut palette = Palette::new();
        palette.index(Bloc::default()); 
        Chunk {
            data: PackedUints::new(size*size*size),
            palette: palette, 
            size, size2: size*size
        }
    }

    pub fn filled(size: usize, bloc: Bloc) -> Self {
        if bloc == Bloc::default() {
            Chunk::new(size)
        } else {
            let mut palette = Palette::new();
            palette.index(Bloc::default());
            palette.index(bloc);
            Chunk {
                data: PackedUints::filled(size*size*size, 1),
                palette: palette, 
                size, size2: size*size
            }
        }
    }
}