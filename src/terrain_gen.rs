use crate::chunk::Chunk;
use std::collections::HashMap;

pub trait TerrainGen: Send + Sync {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized;

    fn gen(&self, col: (i32, i32)) -> HashMap<i32, Chunk>;
}
