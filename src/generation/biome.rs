use noise_algebra::{NoiseSource, Signal2d};
use serde::Deserialize;
use strum_macros::EnumString;

use crate::Block;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
#[derive(EnumString)]
pub enum Biome {
    Desert,
    Savanna,
    Jungle,
    Grassland,
    TemperateForest,
    Marsh,
    Taiga,
    Tundra
}

pub enum Height {
    Const(i32),
    Noise(Signal2d)
}

impl From<i32> for Height {
    fn from(value: i32) -> Self {
        todo!()
    }
}

impl From<Signal2d> for Height {
    fn from(value: Signal2d) -> Self {
        todo!()
    }
}

pub struct Layer {
    height: Height,
    block: Block
}

impl Layer {
    pub fn new<T: Into<Height>>(height: T, block: Block) -> Self {
        Self { height: height.into(), block }
    }
}

pub struct BiomeGen {
    pub height_mod: Height,
    pub layers: Vec<Layer>
}

impl Biome {
    pub fn custom_gen(&self, n: &mut NoiseSource<2>) -> Option<BiomeGen> {
        match self {
            Biome::Desert => Some(BiomeGen {
                height_mod: Height::Const(0),
                layers: vec![Layer::new(n.ridge(3.)*0.1, Block::Sand)]
            }),
            _ => None
        }
    }
}