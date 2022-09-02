use crate::{
    chunk::{Chunk, CHUNK_S1},
    realm::Realm,
};
use array_macro::array;
use std::{collections::HashMap, fmt::Debug, iter, ops::IndexMut};
use strum::{EnumCount, IntoEnumIterator};
pub const MAX_HEIGHT: usize = 256;

pub struct ChunkMap<V = Chunk> {
    chunks: [HashMap<(i32, i32), [Option<V>; MAX_HEIGHT / CHUNK_S1]>; Realm::COUNT],
}

impl<V: Debug> ChunkMap<V> {
    pub fn new() -> Self {
        ChunkMap {
            chunks: array![_ => HashMap::new(); Realm::COUNT],
        }
    }

    pub fn get(&self, realm: Realm, x: i32, y: i32, z: i32) -> Option<&V> {
        self.chunks[realm as usize].get(&(x, z))?[y as usize].as_ref()
    }

    pub fn get_mut(&mut self, realm: Realm, x: i32, y: i32, z: i32) -> Option<&mut V> {
        self.chunks[realm as usize].get_mut(&(x, z))?[y as usize].as_mut()
    }

    pub fn insert(&mut self, realm: Realm, x: i32, y: i32, z: i32, v: V)
    where
        V: Default,
    {
        self.chunks[realm as usize]
            .entry((x, z))
            .or_insert(array![_ => None; MAX_HEIGHT/CHUNK_S1])
            .as_mut()[y as usize] = Some(v);
    }

    pub fn insert_col(&mut self, realm: Realm, x: i32, z: i32, col: [Option<V>; MAX_HEIGHT / CHUNK_S1]) {
        self.chunks[realm as usize].insert((x, z), col);
    }

    pub fn remove_col(&mut self, realm: Realm, x: i32, z: i32) {
        self.chunks[realm as usize].remove(&(x, z));
    }

    pub fn extend(&mut self, other: ChunkMap<V>) {
        for (i, chunks) in other.chunks.into_iter().enumerate() {
            self.chunks[i].extend(chunks);
        }
    }

    pub fn col(&self, realm: Realm, x: i32, z: i32) -> &[Option<V>; MAX_HEIGHT / CHUNK_S1] {
        self.chunks[realm as usize].get(&(x, z)).unwrap()
    }

    pub fn cols(
        &self,
    ) -> impl Iterator<Item = ((Realm, i32, i32), &[Option<V>; MAX_HEIGHT / CHUNK_S1])> {
        Realm::iter().flat_map(|realm| {
            self.chunks[realm as usize]
                .iter()
                .map(move |((x, z), v)| ((realm, *x, *z), v))
        })
    }
}
