use dashmap::DashMap;
use noise::{SuperSimplex, Seedable, NoiseFn};

pub struct NoiseSource {
    sources: DashMap<u32, SuperSimplex>
}

impl NoiseSource {
    pub fn new() -> Self {
        NoiseSource { sources: DashMap::new() }
    }
}


impl NoiseSource {
    pub fn get(&self, seed: u32, x: f64, y: f64) -> f64 {
        if !self.sources.contains_key(&seed) {
            self.sources.insert(seed, SuperSimplex::new().set_seed(seed));
        }
        self.sources.get(&seed).unwrap().get([x, y])
    }
}