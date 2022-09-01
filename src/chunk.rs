use crate::bloc::Bloc;
use crate::get_set::GetSet;
use crate::packed_ints::PackedUsizes;
use serde::{Deserialize, Serialize};
pub const CHUNK_S1: usize = 16;
const CHUNK_S2: usize = CHUNK_S1.pow(2);
const CHUNK_S3: usize = CHUNK_S2.pow(3);

fn index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_S1 + z * CHUNK_S2
}

trait Palette<E> {
    fn index(&mut self, elem: E) -> usize;
}

impl<E: Eq> Palette<E> for Vec<E> {
    fn index(&mut self, elem: E) -> usize {
        self.iter().position(|other| *other == elem).unwrap_or({
            // bloc is not present in the palette
            self.push(elem);
            self.len() - 1
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk<L: GetSet<usize> = PackedUsizes> {
    data: L,
    palette: Vec<Bloc>,
}

impl<L: GetSet<usize>> Chunk<L> {
    pub fn get(&self, x: usize, y: usize, z: usize) -> &Bloc {
        &self.palette[self.data.get(index(x, y, z))]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, bloc: Bloc) {
        let idx = index(x, y, z);
        let value = self.palette.index(bloc);
        self.data.set(idx, value);
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

    pub fn filled(bloc: Bloc) -> Self {
        if bloc == Bloc::Air {
            Chunk::<Vec<usize>>::new()
        } else {
            Chunk {
                data: vec![1; CHUNK_S3],
                palette: vec![Bloc::Air, bloc],
            }
        }
    }
}

fn find_bitsize(len: usize) -> u32 {
    let mut bitsize = 4;
    while 2_u32.pow(bitsize) < len as u32 {
        bitsize += 2;
    }
    bitsize
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