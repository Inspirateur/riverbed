use crate::{generation::{biome_params::BiomeParameters, layer::*}, world::{unchunked, ColPos, CHUNK_S1, CHUNK_S2, WATER_H}, Block};
use strum_macros::EnumString;
use riverbed_noise::*;

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Biome {
    PolarOcean,
    Ocean,
    Plain,
    Mountain,
    Desert
}

impl Biome {
    pub fn generate(&self, seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        match self {
            Biome::PolarOcean => Biome::generate_polar_ocean(seed, col, params),
            Biome::Ocean => Biome::generate_ocean(seed, col, params),
            Biome::Plain => Biome::generate_plain(seed, col, params),
            Biome::Mountain => Biome::generate_mountain(seed, col, params),
            Biome::Desert => Biome::generate_desert(seed, col, params),
            _ => Vec::new(),
        }
    }

    fn generate_polar_ocean(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
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
            Layer { block: Block::Sand, height: Height::Constant(5.), tag: LayerTag::Soil },
            Layer { block: Block::SeaBlock, height: Height::Noise(icebottom), tag: LayerTag::Fixed { height: WATER_H as usize } },
            Layer { block: Block::Ice, height: Height::Noise(icecap), tag: LayerTag::Fixed { height: WATER_H as usize } },
        ]
    }

    fn generate_ocean(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        vec![
            Layer { block: Block::Sand, height: Height::Constant(5.), tag: LayerTag::Soil },
            Layer { block: Block::SeaBlock, height: Height::Constant(WATER_H as f32), tag: LayerTag::Fixed { height: WATER_H as usize } }, 
        ]
    }

    fn generate_plain(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let plain = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed, 
            0.001, 
            WATER_H as f32 + 3., 
            WATER_H as f32 + 7.
        );
        vec![
            Layer { block: Block::Granite, height: Height::Constant(WATER_H as f32), tag: LayerTag::Mantle },
            Layer { block: Block::GrassBlock, height: Height::Noise(plain), tag: LayerTag::Soil }, 
        ]
    }

    fn generate_mountain(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let n = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed, 
            0.02, 
            WATER_H as f32, 
            WATER_H as f32 + 150.
        );
        vec![
            Layer { block: Block::Granite, height: Height::Noise(n), tag: LayerTag::Mantle }
        ]
    }

    fn generate_desert(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let mut sin = vec![0.0; CHUNK_S2];
        for dx in 0..CHUNK_S1 {
            for dy in 0..CHUNK_S1 {
                let index = dx + dy * CHUNK_S1;
                sin[index] = (unchunked(col.x, dx) as f32 /6.).sin() * 4.0 + 8. + WATER_H as f32;
            }
        }
        vec![
            Layer { block: Block::Granite, height: Height::Constant(WATER_H as f32), tag: LayerTag::Mantle },
            Layer { block: Block::Sand, height: Height::Noise(sin), tag: LayerTag::Deposit }, 
        ]
    }
}
