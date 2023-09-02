use crate::{earth_gen::Earth, debug_gen::DebugGen};
use ourcraft::{Realm, ChunkPos2D, Blocs};
use bevy::prelude::Resource;
use dashmap::DashMap;
use std::collections::HashMap;

pub trait TerrainGen: Send + Sync {
    fn set_config(&mut self, config: HashMap<String, f32>);

    fn set_seed(&mut self, seed: u32);

    fn gen(&self, world: &mut Blocs, col: ChunkPos2D);
}

#[derive(Resource)]
pub struct Generators {
    data: DashMap<Realm, Box<dyn TerrainGen>>,
}

impl Generators {
    pub fn new(seed: u32) -> Self {
        let gens: DashMap<Realm, Box<dyn TerrainGen>> = DashMap::new();
        gens.insert(Realm::Overworld, Box::new(DebugGen::new(seed, HashMap::new())));
        Generators { data: gens }
    }

    pub fn gen(&self, world: &mut Blocs, pos: ChunkPos2D) {
        self.data.get(&Realm::Overworld).unwrap().gen(world, pos)
    }
}
