use crate::{biome_params::BiomeParameters, layer::*};
use rb_block::Block;
use rb_noise::*;
use rb_world::{CHUNK_S1, CHUNK_S2, ChunkPos2d, WATER_H, unchunked};
use strum_macros::EnumString;
const MOUNTAIN_H: f32 = 150.;

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
    pub fn generate(&self, seed: u32, col: ChunkPos2d, params: &BiomeParameters) -> Vec<Layer> {
        match self {
            Biome::PolarOcean => Biome::generate_polar_ocean(seed, col, params),
            Biome::Ocean => Biome::generate_ocean(seed, col, params),
            Biome::Mountain => Biome::generate_mountain(seed, col, params),
            Biome::Desert => Biome::generate_desert(seed, col, params),
            Biome::Jungle => Biome::generate_jungle(seed, col, params),
            Biome::Canyon => Biome::generate_canyon(seed, col, params),
            Biome::Tundra => Biome::generate_tundra(seed, col, params),
            // Plain is the default for any not yet implemented biomes
            _ => Biome::generate_plain(seed, col, params),
        }
    }

    fn generate_polar_ocean(seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = col.to_real_pos();
        let mut n = ridge(x, CHUNK_S1, z, CHUNK_S1, seed, 0.05);
        powi(&mut n, 3);
        mul_const(&mut n, -2.);
        add_const(&mut n, WATER_H as f32);
        vec![
            Layer {
                block: Block::Sand,
                height: Height::Constant(5.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::SeaBlock,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Fixed {
                    height: WATER_H as usize,
                },
            },
            Layer {
                block: Block::Ice,
                height: Height::Noise(n),
                tag: LayerTag::Fixed {
                    height: (WATER_H as usize) - 1,
                },
            },
        ]
    }

    fn generate_ocean(_seed: u32, _col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        vec![
            Layer {
                block: Block::Sand,
                height: Height::Constant(5.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::SeaBlock,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Fixed {
                    height: WATER_H as usize,
                },
            },
        ]
    }

    fn generate_plain(seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = col.to_real_pos();
        let mut plain = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 10, 0.08);
        let mut mask: Vec<f32> = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 11, 0.005);
        points_lerp(&mut mask, &[(0., 0.), (0.4, 0.1), (0.6, 0.9), (1., 1.)]);
        mul(&mut plain, &mask);
        mul_const(&mut plain, 30.);
        add_const(&mut plain, WATER_H as f32 + 15.);
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(plain),
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_mountain(seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = col.to_real_pos();
        let mut n = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 10, 0.03);
        let mut top = n.clone();
        let hills = fbm_scaled(
            x,
            CHUNK_S1,
            z,
            CHUNK_S1,
            seed + 11,
            0.05,
            WATER_H as f32 + 5.,
            WATER_H as f32 + 10.,
        );
        points_lerp(
            &mut n,
            &[
                (0., WATER_H as f32),
                (0.6, WATER_H as f32 + 5.),
                (0.9, MOUNTAIN_H - 5.),
                (1., MOUNTAIN_H),
            ],
        );
        points_lerp(
            &mut top,
            &[
                (0., 0.),
                (0.6, 0.),
                (0.9, MOUNTAIN_H - 5.),
                (1., MOUNTAIN_H),
            ],
        );
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Noise(n),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(hills),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::Snow,
                height: Height::Noise(top),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_desert(_seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let mut sin = vec![0.0; CHUNK_S2];
        for dx in 0..CHUNK_S1 {
            for dy in 0..CHUNK_S1 {
                let index = dx + dy * CHUNK_S1;
                sin[index] = (unchunked::<CHUNK_S1, 1>(col.x, dx) as f32 / 6.).sin() * 4.0
                    + 8.
                    + WATER_H as f32;
            }
        }
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Sand,
                height: Height::Noise(sin),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_jungle(seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = col.to_real_pos();
        let mut n = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 10, 0.1);
        let mask = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 11, 0.1);
        mul(&mut n, &mask);
        mul_const(&mut n, 60.);
        add_const(&mut n, WATER_H as f32 + 5.);
        quantize(&mut n, 4.);
        let mut granite = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 12, 0.1);
        points_lerp(
            &mut granite,
            &[
                (0., WATER_H as f32),
                (0.8, WATER_H as f32),
                (0.9, WATER_H as f32 + 20.),
                (1., WATER_H as f32 + 25.),
            ],
        );
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Noise(granite),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Podzol,
                height: Height::Constant(WATER_H as f32 + 15.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(n),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_canyon(seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = col.to_real_pos();
        let mut n = ridge(x, CHUNK_S1, z, CHUNK_S1, seed + 10, 0.01);
        mul_const(&mut n, -1.);
        add_const(&mut n, 1.);
        let mut top = n.clone();
        points_lerp(
            &mut n,
            &[
                (0., WATER_H as f32 + 5.),
                (0.35, WATER_H as f32 + 10.),
                (0.45, MOUNTAIN_H - 5.),
                (1., MOUNTAIN_H),
            ],
        );
        points_lerp(
            &mut top,
            &[
                (0., 0.),
                (0.4, 0.),
                (0.45, MOUNTAIN_H - 5.),
                (1., MOUNTAIN_H),
            ],
        );
        vec![
            Layer {
                block: Block::CoarseDirt,
                height: Height::Noise(n),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Sand,
                height: Height::Constant(WATER_H as f32 + 8.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(top),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_tundra(seed: u32, col: ChunkPos2d, _params: &BiomeParameters) -> Vec<Layer> {
        let (x, z) = col.to_real_pos();
        let n = fbm_scaled(
            x,
            CHUNK_S1,
            z,
            CHUNK_S1,
            seed + 10,
            0.04,
            WATER_H as f32 + 15.,
            WATER_H as f32 + 30.,
        );
        let mut icicles = fbm(x, CHUNK_S1, z, CHUNK_S1, seed + 11, 0.1);
        powi(&mut icicles, 6);
        mul_const(&mut icicles, 60.);
        add_const(&mut icicles, WATER_H as f32 + 5.);
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Snow,
                height: Height::Noise(n),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::Ice,
                height: Height::Noise(icicles),
                tag: LayerTag::Deposit,
            },
        ]
    }
}
