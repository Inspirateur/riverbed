use crate::{
    bloc::Bloc,
    bloc_pos::{BlocPos, BlocPos2D, BlocPosChunked, BlocPosChunked2D, ChunkPos, ChunkPos2D},
    chunk::{Chunk, CHUNK_S1},
    realm::Realm,
};
use array_macro::array;
use std::collections::HashMap;
use strum::{EnumCount, IntoEnumIterator};
pub const MAX_HEIGHT: usize = 256;

pub struct Blocs {
    chunks: [HashMap<(i32, i32), [Option<Chunk>; MAX_HEIGHT / CHUNK_S1]>; Realm::COUNT],
}

impl Blocs {
    pub fn new() -> Self {
        Blocs {
            chunks: array![_ => HashMap::new(); Realm::COUNT],
        }
    }

    pub fn get(&self, pos: BlocPos) -> Bloc {
        let pos = BlocPosChunked::from(pos);
        match self.get_chunk(pos.chunk) {
            Some(chunk) => *chunk.get(pos.dx, pos.dy, pos.dz),
            None => Bloc::Air,
        }
    }

    pub fn top(&self, pos: BlocPos2D) -> (Bloc, i32) {
        let pos = BlocPosChunked2D::from(pos);
        let col = self.chunks[pos.col.realm as usize]
            .get(&(pos.col.x, pos.col.z))
            .unwrap();
        for cy in (0..col.len()).rev() {
            if let Some(chunk) = &col[cy] {
                for dy in (0..CHUNK_S1).rev() {
                    let bloc = chunk.get(pos.dx, dy, pos.dz);
                    if *bloc != Bloc::Air {
                        return (*bloc, (cy * CHUNK_S1 + dy) as i32);
                    }
                }
            }
        }
        (Bloc::Bedrock, 0)
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks[pos.realm as usize].get(&(pos.x, pos.z))?[pos.y as usize].as_ref()
    }

    pub fn insert_col(&mut self, pos: ChunkPos2D, col: [Option<Chunk>; MAX_HEIGHT / CHUNK_S1]) {
        self.chunks[pos.realm as usize].insert((pos.x, pos.z), col);
    }

    pub fn remove_col(&mut self, pos: ChunkPos2D) -> bool {
        self.chunks[pos.realm as usize]
            .remove(&(pos.x, pos.z))
            .is_some()
    }

    pub fn contains_col(&self, pos: ChunkPos2D) -> bool {
        self.chunks[pos.realm as usize].contains_key(&(pos.x, pos.z))
    }

    pub fn extend(&mut self, other: Blocs) {
        for (i, chunks) in other.chunks.into_iter().enumerate() {
            self.chunks[i].extend(chunks);
        }
    }

    pub fn cols(
        &self,
    ) -> impl Iterator<Item = (ChunkPos2D, &[Option<Chunk>; MAX_HEIGHT / CHUNK_S1])> {
        Realm::iter().flat_map(|realm| {
            self.chunks[realm as usize].iter().map(move |((x, z), c)| {
                (
                    ChunkPos2D {
                        realm,
                        x: *x,
                        z: *z,
                    },
                    c,
                )
            })
        })
    }
}
