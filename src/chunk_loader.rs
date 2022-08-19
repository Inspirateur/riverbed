use crate::bloc::Bloc;
use crate::packed_ints::PackedUsizes;
use crate::terrain::Earth;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
const CHUNK_S1: usize = 16;
const CHUNK_S2: usize = CHUNK_S1.pow(2);
const CHUNK_S3: usize = CHUNK_S2.pow(3);

#[derive(Serialize, Deserialize)]
struct Chunk {
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

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_S2 + z * CHUNK_S3
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> &Bloc {
        &self.palette[self.data.get(Chunk::index(x, y, z))]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, bloc: Bloc) {
        let idx = Chunk::index(x, y, z);
        let value = self.palette.iter().position(|b| *b == bloc).unwrap_or({
            // bloc is not present in the palette
            self.palette.push(bloc);
            self.palette.len() - 1
        });
        self.data.set(idx, value);
    }
}

struct ChunkLoader {
    worldgen: Earth,
    chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl ChunkLoader {
    pub fn new(worldgen: Earth) -> Self {
        ChunkLoader {
            worldgen,
            chunks: HashMap::new(),
        }
    }
}
