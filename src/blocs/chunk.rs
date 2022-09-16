use crate::pos::ChunkedPos;
use crate::{blocs::bloc::Bloc, packed_ints::find_bitsize};
use crate::get_set::GetSet;
use crate::packed_ints::PackedUsizes;
use crate::utils::palette::Palette;
use serde::{Deserialize, Serialize};
pub const CHUNK_S1: usize = 32;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
const CHUNK_S3: usize = CHUNK_S1.pow(3);

fn index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_S1 + z * CHUNK_S2
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk<L: GetSet<usize> = PackedUsizes> {
    data: L,
    palette: Vec<Bloc>,
}

impl<L: GetSet<usize>> Chunk<L> {
    pub fn get(&self, (x, y, z): ChunkedPos) -> &Bloc {
        &self.palette[self.data.get(index(x, y, z))]
    }

    pub fn set(&mut self, (x, y, z): ChunkedPos, bloc: Bloc) {
        let idx = index(x, y, z);
        self.data.set(idx, self.palette.index(bloc));
    }

    pub fn set_if_empty(&mut self, (x, y, z): ChunkedPos, bloc: Bloc) -> bool {
        let idx = index(x, y, z);
        if self.palette[self.data.get(idx)] != Bloc::Air {
            return false;
        }
        self.data.set(idx, self.palette.index(bloc));
        true
    }
}

impl Chunk<PackedUsizes> {
    pub fn new() -> Self {
        Chunk {
            data: PackedUsizes::new(CHUNK_S3, 4),
            palette: vec![Bloc::Air],
        }
    }

    pub fn filled(bloc: Bloc) -> Self {
        if bloc == Bloc::Air {
            Chunk::<PackedUsizes>::new()
        } else {
            Chunk {
                data: PackedUsizes::filled(CHUNK_S3, 4, 1),
                palette: vec![Bloc::Air, bloc],
            }
        }
    }
}

impl Chunk<Vec<usize>> {
    pub fn new() -> Self {
        Chunk {
            data: vec![0; CHUNK_S3],
            palette: vec![Bloc::Air],
        }
    }
}

impl From<Chunk<Vec<usize>>> for Chunk<PackedUsizes> {
    fn from(chunk: Chunk<Vec<usize>>) -> Self {
        Chunk {
            data: PackedUsizes::from_usizes(chunk.data, find_bitsize(chunk.palette.len())),
            palette: chunk.palette,
        }
    }
}

impl Default for Chunk<Vec<usize>> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Chunk<PackedUsizes> {
    fn default() -> Self {
        Self::new()
    }
}
