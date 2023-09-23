use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use bevy::prelude::Resource;
use indexmap::IndexMap;
use crate::{ChunkedPos, Chunk, ChunkPos, Y_CHUNKS, Pos, ChunkedPos2D, chunked};
use crate::bloc::Bloc;
use super::pos::{ChunkPos2D, BlocPos, BlocPos2D};
use super::{CHUNK_S1, chunk};

pub enum ChunkChanges {
    Created,
    Edited(Vec<(ChunkedPos, Bloc)>)
}

impl ChunkChanges {
    pub fn new(new: bool) -> Self {
        if new {
            ChunkChanges::Created
        } else {
            ChunkChanges::Edited(Vec::new())
        }
    }

    pub fn push(&mut self, chunked_pos: ChunkedPos, bloc: Bloc) {
        match self {
            ChunkChanges::Created => (),
            ChunkChanges::Edited(ref mut changes) => changes.push((chunked_pos, bloc))
        }
    }
}

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
    // using index map because we want to preserve insertion order here
    pub changes: IndexMap<ChunkPos, ChunkChanges>,
    pub tracking: HashSet<ChunkPos>,
}

impl Blocs {
    pub fn new() -> Self {
        Blocs {
            chunks: HashMap::new(),
            changes: IndexMap::new(),
            tracking: HashSet::new(),
        }
    }

    pub fn set_bloc(&mut self, pos: BlocPos, bloc: Bloc) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        if self.tracking.contains(&chunk_pos) {
            self.changes.entry(chunk_pos).or_insert_with(
                || ChunkChanges::new(!self.chunks.contains_key(&chunk_pos))
            ).push(chunked_pos, bloc);    
        }
        self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set(chunked_pos, bloc);
    }

    pub fn set_chunked(&mut self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos, bloc: Bloc) {
        // BYPASSES CHANGE DETECTION, used by terrain generation for efficiency
        self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set(chunked_pos, bloc);
    }

    pub fn set_yrange(&mut self, chunk_pos2d: ChunkPos2D, (x, z): ChunkedPos2D, top: i32, mut height: usize, bloc: Bloc) {
        // BYPASSES CHANGE DETECTION, used by terrain generation to efficiently fill columns of blocs
        let (mut cy, mut dy) = chunked(top);
        while height > 0 && cy >= 0 {
            let chunk_pos = ChunkPos { x: chunk_pos2d.x, y: cy, z: chunk_pos2d.z, realm: chunk_pos2d.realm};
            let h = height.min(dy);
            self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set_yrange((x, dy, z), h, bloc);
            height -= h;
            cy -= 1;
            dy = CHUNK_S1-1;
        }
    }

    pub fn set_if_empty(&mut self, pos: BlocPos, bloc: Bloc) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        let new_chunk = !self.chunks.contains_key(&chunk_pos);
        if self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(CHUNK_S1)).set_if_empty(chunked_pos, bloc) 
            && self.tracking.contains(&chunk_pos) 
        {
            self.changes.entry(chunk_pos).or_insert_with(|| ChunkChanges::new(new_chunk)).push(
                chunked_pos, bloc
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

    pub fn is_col_loaded(&self, player_pos: Pos<f32>) -> bool {
        let (chunk_pos, _): (Pos<i32>, _) = <BlocPos>::from(player_pos).into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk = Pos { x: chunk_pos.x, y, z: chunk_pos.z, realm: chunk_pos.realm };
            if self.chunks.contains_key(&chunk) {
                return true;
            }
        }
        false
    }

    pub fn register(&mut self, col: ChunkPos2D) {
        // Used by terrain generation to batch register chunks for efficiency
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {x: col.x, y, z: col.z, realm: col.realm };
            self.tracking.insert(chunk_pos);
            if self.chunks.contains_key(&chunk_pos) {
                self.changes.insert(chunk_pos, ChunkChanges::Created);
            }
        }
    }
    
    pub fn unload_col(&mut self, col: ChunkPos2D) {
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {x: col.x, y, z: col.z, realm: col.realm };
            self.chunks.remove(&chunk_pos);
            self.changes.remove(&chunk_pos);
            self.tracking.remove(&chunk_pos);
        }
    }
}