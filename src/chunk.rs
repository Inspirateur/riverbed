use crate::bloc::Bloc;
use crate::earth_gen::Earth;
use crate::packed_ints::PackedUsizes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub const CHUNK_S1: usize = 16;
const CHUNK_S2: usize = CHUNK_S1.pow(2);
const CHUNK_S3: usize = CHUNK_S2.pow(3);

fn index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_S2 + z * CHUNK_S3
}

pub struct RawChunk {
    data: Vec<Bloc>,
}

impl RawChunk {
    pub fn new() -> Self {
        RawChunk {
            data: vec![Bloc::Air; CHUNK_S3],
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> &Bloc {
        &self.data[index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, bloc: Bloc) {
        self.data[index(x, y, z)] = bloc
    }
}

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    data: PackedUsizes,
    palette: Vec<Bloc>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            data: PackedUsizes::new(CHUNK_S3, 4),
            palette: vec![Bloc::Air],
        }
    }

    pub fn filled(bloc: Bloc) -> Self {
        if bloc == Bloc::Air {
            Chunk::new()
        } else {
            Chunk {
                data: PackedUsizes::filled(CHUNK_S3, 4, 1),
                palette: vec![Bloc::Air, bloc],
            }
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> &Bloc {
        &self.palette[self.data.get(index(x, y, z))]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, bloc: Bloc) {
        let idx = index(x, y, z);
        let value = self.palette.iter().position(|b| *b == bloc).unwrap_or({
            // bloc is not present in the palette
            self.palette.push(bloc);
            self.palette.len() - 1
        });
        self.data.set(idx, value);
    }
}

impl From<RawChunk> for Chunk {
    fn from(_: RawChunk) -> Self {
        todo!()
    }
}
