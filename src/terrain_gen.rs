use crate::{
    pos::{ChunkPos2D, Realm},
    blocs::Col,
    earth_gen::Earth,
};
use dashmap::DashMap;
use std::collections::HashMap;

pub trait TerrainGen: Send + Sync {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized + Clone;

    fn gen(&self, col: (i32, i32)) -> Col;
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

    pub fn gen(&self, pos: ChunkPos2D) -> Col {
        self.data.get(&pos.realm).unwrap().gen((pos.x, pos.z))
    }
}
