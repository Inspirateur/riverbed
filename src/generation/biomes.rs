use crate::{generation::biome_params::BiomeParameters, world::{unchunked, ColPos, CHUNK_S1, CHUNK_S2, WATER_H}, Block};
use strum_macros::{EnumIter, EnumString};
use riverbed_noise::*;


#[derive(Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum LayerTag {
    Mantle,
    Soil, 
    Water,
    Structure {
        base_height: usize
    }
}

pub enum Layer {
    Constant(f32),
    Noise(Vec<f32>),
}

impl Layer {
    pub fn height(&self, dx: usize, dz: usize) -> f32 {
        match self {
            Layer::Constant(h) => *h,
            Layer::Noise(noise) => noise[dx + dz * CHUNK_S1],
        }
    }
}

#[derive(Clone, Copy, EnumString, PartialEq, Eq, Hash)]
pub enum Biome {
    PolarOcean,
    Ocean,
    Plain,
    Mountain,
    Desert
}

impl Biome {
    pub fn generate(&self, seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<(Block, Layer, LayerTag)> {
        match self {
            Biome::PolarOcean => Biome::generate_polar_ocean(seed, col, params),
            Biome::Ocean => Biome::generate_ocean(seed, col, params),
            Biome::Plain => Biome::generate_plain(seed, col, params),
            Biome::Mountain => Biome::generate_mountain(seed, col, params),
            Biome::Desert => Biome::generate_desert(seed, col, params),
            _ => Vec::new(),
        }
    }

    fn generate_polar_ocean(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<(Block, Layer, LayerTag)> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let minice = WATER_H as f32 - 2.;
        let icecap = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed, 
            0.01, 
            minice, 
            WATER_H as f32 + 5.0
        );
        let icebottom = icecap.iter().map(|v| WATER_H as f32 - (v-minice)*5.).collect();
        vec![
            (Block::Ice, Layer::Noise(icecap), LayerTag::Structure { base_height: WATER_H as usize }),
            (Block::SeaBlock, Layer::Noise(icebottom), LayerTag::Water),
            (Block::Sand, Layer::Constant(5.), LayerTag::Soil),
        ]
    }

    fn generate_ocean(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<(Block, Layer, LayerTag)> {
        vec![(Block::SeaBlock, Layer::Constant(WATER_H as f32), LayerTag::Water), (Block::Sand, Layer::Constant(5.), LayerTag::Soil)]
    }

    fn generate_plain(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<(Block, Layer, LayerTag)> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let plain = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed, 
            0.001, 
            WATER_H as f32 + 1., 
            WATER_H as f32 + 3.
        );
        vec![(Block::GrassBlock, Layer::Noise(plain), LayerTag::Soil), (Block::Granite, Layer::Constant(WATER_H as f32), LayerTag::Mantle)]
    }

    fn generate_mountain(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<(Block, Layer, LayerTag)> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let n = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed, 
            0.05, 
            WATER_H as f32, 
            WATER_H as f32 + 100.
        );
        vec![(Block::Granite, Layer::Noise(n), LayerTag::Mantle)]
    }

    fn generate_desert(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<(Block, Layer, LayerTag)> {
        let mut sin = vec![0.0; CHUNK_S2];
        for dx in 0..CHUNK_S1 {
            for dy in 0..CHUNK_S1 {
                let index = dx + dy * CHUNK_S1;
                sin[index] = (unchunked(col.x, dx) as f32 /6.).sin() * 4.0 + 8. + WATER_H as f32;
            }
        }
        vec![(Block::Sand, Layer::Noise(sin), LayerTag::Soil), (Block::Granite, Layer::Constant(WATER_H as f32), LayerTag::Mantle)]
    }
}
