use std::collections::HashMap;
use crate::{chunk::{Chunk, CHUNK_S1}, earth_gen::Earth, realm::Realm, blocs::MAX_HEIGHT};
use dashmap::DashMap;

pub trait TerrainGen: Send + Sync {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where Self: Sized + Clone;

    fn gen(&self, col: (i32, i32)) -> [Option<Chunk>; MAX_HEIGHT / CHUNK_S1];
}

pub fn generators(seed: u32) -> DashMap<Realm, Box<dyn TerrainGen>> {
    let gens: DashMap<Realm, Box<dyn TerrainGen>> = DashMap::new();
    gens.insert(Realm::Earth, Box::new(Earth::new(seed, HashMap::new())));
    gens
}
