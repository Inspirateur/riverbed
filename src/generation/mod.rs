mod range_utils;
mod tree;
mod growables;
mod coverage;
mod biome_params;
mod plant_params;
mod layer;
mod biomes;
mod terrain;
use std::ops::Range;
use crate::Block;
pub use terrain::TerrainGenerator;

type Soils = Vec<([Range<f32>; 2], Block)>;
