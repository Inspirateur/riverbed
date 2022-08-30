use crate::{chunk::Chunk, earth_gen::Earth, realm::Realm};
use std::collections::HashMap;

pub trait TerrainGen: Send + Sync {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized;

    fn gen(&self, col: (i32, i32)) -> HashMap<i32, Chunk>;
}

pub fn generators(seed: u32) -> HashMap<Realm, Box<dyn TerrainGen>> {
    let mut gens: HashMap<Realm, Box<dyn TerrainGen>> = HashMap::new();
    gens.insert(Realm::Earth, Box::new(Earth::new(seed, HashMap::new())));
    gens
}
