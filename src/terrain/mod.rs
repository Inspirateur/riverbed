mod terrain_gen;
mod debug_gen;
mod earth_gen;
mod tree;
mod biome;
mod growables;

pub use terrain_gen::setup_gen_thread;

use std::ops::Range;
use crate::Block;

type Soils = Vec<([Range<f32>; 2], Block)>;
