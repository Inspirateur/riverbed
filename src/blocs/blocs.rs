use std::collections::HashMap;
use std::hash::Hash;
use bevy::prelude::Resource;
use crate::{ChunkedPos, Chunk, ChunkPos, Y_CHUNKS};
use crate::bloc::Bloc;
use super::pos::{ChunkPos2D, BlocPos, BlocPos2D};
use super::CHUNK_S1;

pub type Cols<E> = HashMap<ChunkPos2D, E>;

pub trait HashMapUtils<K, V> {
    fn pop(&mut self) -> Option<(K, V)>;
}

impl<K: Eq + PartialEq + Hash + Clone, V> HashMapUtils<K, V> for HashMap<K, V> {
    fn pop(&mut self) -> Option<(K, V)> {
        let key = self.keys().next().cloned()?;
        let value = self.remove(&key)?;
        Some((key, value))
    }
}

#[derive(Resource)]
pub struct Blocs {
    pub chunks: HashMap<ChunkPos, Chunk>,
    pub changes: HashMap<ChunkPos, Vec<(ChunkedPos, Bloc)>>
}

impl Blocs {
    pub fn new() -> Self {
        Blocs {
            chunks: HashMap::new(),
            changes: HashMap::new()
        }
    }

    pub fn set_bloc(&mut self, pos: BlocPos, bloc: Bloc) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        self.changes.entry(chunk_pos).or_insert_with(Vec::new).push(
            (chunked_pos, bloc)
        );
        self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set(chunked_pos, bloc);
    }

    pub fn set_chunked(&mut self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos, bloc: Bloc) {
        // EXEMPT FROM CHANGE TRACKING, used by generation
        self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set(chunked_pos, bloc);
    }

    pub fn set_if_empty(&mut self, pos: BlocPos, bloc: Bloc) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        if self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set_if_empty(chunked_pos, bloc) {
            self.changes.entry(chunk_pos).or_insert_with(Vec::new).push(
                (chunked_pos, bloc)
            );
        }
    }
    
    pub fn get_block(&self, pos: BlocPos) -> Bloc {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        match self.chunks.get(&chunk_pos) {
            None => Bloc::default(),
            Some(chunk) => chunk.get(chunked_pos).clone()
        }
    }

    pub fn top_block(&self, pos: BlocPos2D) -> (Bloc, i32) {
        let (col_pos, pos2d) = pos.into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk_pos = ChunkPos {
                x: col_pos.x,
                y,
                z: col_pos.z,
                realm: col_pos.realm
            };
            if let Some(chunk) = self.chunks.get(&chunk_pos) {
                let (bloc, bloc_y) = chunk.top(pos2d);
                if *bloc != Bloc::default() {
                    return (bloc.clone(), y*CHUNK_S1 as i32 + bloc_y as i32);
                }
            }
        }
        (Bloc::default(), 0)
    }

    pub fn unload_col(&mut self, col: ChunkPos2D) {
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {x: col.x, y, z: col.z, realm: col.realm };
            self.chunks.remove(&chunk_pos);
            self.changes.remove(&chunk_pos);
        }
    }
}