mod earth_gen;
mod tree;
mod growables;
mod biome_params;
mod layer;
mod biomes;
mod terrain;
use std::ops::Range;
use crate::Block;
pub use earth_gen::Earth;
pub use terrain::TerrainGenerator;

type Soils = Vec<([Range<f32>; 2], Block)>;
