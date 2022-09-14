use dashmap::DashMap;
use noise::{Perlin, Seedable, NoiseFn};

pub struct NoiseSource {
    sources: DashMap<u32, Perlin>
}

impl NoiseSource {
    pub fn new() -> Self {
        NoiseSource { sources: DashMap::new() }
    }
}


impl NoiseSource {
    pub fn get(&self, seed: u32, x: f64, y: f64) -> f64 {
        if !self.sources.contains_key(&seed) {
            self.sources.insert(seed, Perlin::new().set_seed(seed));
        }
        self.sources.get(&seed).unwrap().get([x, y])
    }
}