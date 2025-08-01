use crate::{generation::{biome_params::BiomeParameters, layer::*}, world::{unchunked, ColPos, CHUNK_S1, CHUNK_S2, WATER_H}, Block};
use strum_macros::EnumString;
use riverbed_noise::*;
const MOUNTAIN_H: f32 = 120.;

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Biome {
    PolarOcean,
    Canyon,
    Ocean,
    Plain,
    Mountain,
    Desert,
    Tundra,
    Savannah,
    Jungle,
}

impl Biome {
    pub fn generate(&self, seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        match self {
            Biome::PolarOcean => Biome::generate_polar_ocean(seed, col, params),
            Biome::Ocean => Biome::generate_ocean(seed, col, params),
            Biome::Mountain => Biome::generate_mountain(seed, col, params),
            Biome::Desert => Biome::generate_desert(seed, col, params),
            Biome::Jungle => Biome::generate_jungle(seed, col, params),
            Biome::Canyon => Biome::generate_canyon(seed, col, params),
            // Plain is the default for any not yet implemented biomes
            _ => Biome::generate_plain(seed, col, params),
        }
    }

    fn generate_polar_ocean(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let mut n = ridge(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed, 
            0.01, 
        );
        powi(&mut n, 4);
        mul_const(&mut n, -2.);
        add_const(&mut n, WATER_H as f32);
        vec![
            Layer { block: Block::Sand, height: Height::Constant(5.), tag: LayerTag::Soil },
            Layer { block: Block::SeaBlock, height: Height::Constant(WATER_H as f32), tag: LayerTag::Fixed { height: WATER_H as usize } },
            Layer { block: Block::Ice, height: Height::Noise(n), tag: LayerTag::Fixed { height: (WATER_H as usize) -1 } },
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
            0.04, 
            WATER_H as f32 + 3., 
            WATER_H as f32 + 10.
        );
        vec![
            Layer { block: Block::Granite, height: Height::Constant(WATER_H as f32), tag: LayerTag::Mantle },
            Layer { block: Block::GrassBlock, height: Height::Noise(plain), tag: LayerTag::Soil }, 
        ]
    }

    fn generate_mountain(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let mut mountain_presence = fbm(x, CHUNK_S1, z, CHUNK_S1, seed+1, 0.005);
        powi(&mut mountain_presence, 2);
        let mut n = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed+2, 
            0.05, 
            WATER_H as f32 + 3., 
            WATER_H as f32 + MOUNTAIN_H
        );
        mul(&mut n, &mountain_presence);
        add_const(&mut mountain_presence, -0.7);
        mul_const(&mut mountain_presence, 5.);
        powi(&mut mountain_presence, 2);
        add(&mut mountain_presence, &n);
        vec![
            Layer { block: Block::Granite, height: Height::Noise(n), tag: LayerTag::Mantle },
            Layer { block: Block::GrassBlock, height: Height::Noise(mountain_presence), tag: LayerTag::Soil }
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

    fn generate_jungle(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let mut n = fbm_scaled(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed+2, 
            0.1, 
            WATER_H as f32, 
            WATER_H as f32 + 60.
        );
        quantize(&mut n, 4.);
        vec![
            Layer { block: Block::Granite, height: Height::Constant(WATER_H as f32), tag: LayerTag::Mantle },
            Layer { block: Block::Podzol, height: Height::Constant(WATER_H as f32 + 15.), tag: LayerTag::Soil },
            Layer { block: Block::GrassBlock, height: Height::Noise(n), tag: LayerTag::Deposit }, 
        ]
    }

    fn generate_canyon(seed: u32, col: ColPos, params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let mut n = ridge(
            x, CHUNK_S1, 
            z, CHUNK_S1, 
            seed+2, 
            0.05, 
        );
        powi(&mut n, 4);
        mul_const(&mut n, -100.);
        add_const(&mut n, 100.);
        vec![
            Layer { block: Block::CoarseDirt, height: Height::Noise(n), tag: LayerTag::Mantle },
        ]
    }
}
