use std::collections::HashMap;
use crate::{chunk::Chunk, earth_gen::Earth, realm::Realm};
use dashmap::DashMap;

pub trait TerrainGen: Send + Sync {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where Self: Sized + Clone;

    fn gen(&self, col: (i32, i32)) -> HashMap<i32, Chunk>;
}

pub fn generators(seed: u32) -> DashMap<Realm, Box<dyn TerrainGen>> {
    let gens: DashMap<Realm, Box<dyn TerrainGen>> = DashMap::new();
    gens.insert(Realm::Earth, Box::new(Earth::new(seed, HashMap::new())));
    gens
}
