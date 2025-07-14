mod earth_gen;
mod tree;
mod biome;
mod growables;
use std::ops::Range;
use crate::Block;
pub use earth_gen::Earth;

type Soils = Vec<([Range<f32>; 2], Block)>;
