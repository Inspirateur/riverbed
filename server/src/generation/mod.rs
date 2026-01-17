mod biome_params;
mod biomes;
mod coverage;
mod growables;
mod layer;
mod plant_params;
mod range_utils;
mod terrain;
mod tree;
use shared::block::Block;
use std::ops::Range;
pub use terrain::TerrainGenerator;

type Soils = Vec<([Range<f32>; 2], Block)>;
