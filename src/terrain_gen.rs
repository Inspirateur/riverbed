use crate::{
    bloc_pos::ChunkPos2D,
    blocs::MAX_HEIGHT,
    chunk::{Chunk, CHUNK_S1},
    earth_gen::Earth,
    realm::Realm,
};
use dashmap::DashMap;
use std::collections::HashMap;

pub trait TerrainGen: Send + Sync {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized + Clone;

    fn gen(&self, col: (i32, i32)) -> [Option<Chunk>; MAX_HEIGHT / CHUNK_S1];
}

pub struct Generators {
    data: DashMap<Realm, Box<dyn TerrainGen>>,
}

impl Generators {
    pub fn new(seed: u32) -> Self {
        let gens: DashMap<Realm, Box<dyn TerrainGen>> = DashMap::new();
        gens.insert(Realm::Earth, Box::new(Earth::new(seed, HashMap::new())));
        Generators { data: gens }
    }

    pub fn gen(&self, pos: ChunkPos2D) -> [Option<Chunk>; MAX_HEIGHT / CHUNK_S1] {
        self.data.get(&pos.realm).unwrap().gen((pos.x, pos.z))
    }
}
