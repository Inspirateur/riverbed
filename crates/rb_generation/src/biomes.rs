use crate::{biome_params::BiomeParameters, layer::*, noise_samples::NoiseSamples, terrain::FREQ};
use quick_noise::{
    Billow, Fbm, Grid, GridGenerator, Octave, Perlin, Ridged, simd::SimdSliceIterExt,
    simd::StaticSimd,
};
use rb_block::Block;
use rb_world::{CHUNK_S1, ChunkPos2d, WATER_H};
use std::ops::{Mul, MulAssign};
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
    pub fn generate(&self, generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        match self {
            Biome::PolarOcean => Biome::generate_polar_ocean(generator, noise_samples),
            Biome::Ocean => Biome::generate_ocean(generator, noise_samples),
            Biome::Mountain => Biome::generate_mountain(generator, noise_samples),
            Biome::Desert => Biome::generate_desert(generator, noise_samples),
            Biome::Jungle => Biome::generate_jungle(generator, noise_samples),
            Biome::Canyon => Biome::generate_canyon(generator, noise_samples),
            Biome::Tundra => Biome::generate_tundra(generator, noise_samples),
            // Plain is the default for any not yet implemented biomes
            _ => Biome::generate_plain(generator, noise_samples),
        }
    }

    fn generate_polar_ocean(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.05)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v
                .mul(*v)
                .mul(*v)
                .mul_add(StaticSimd::splat(-2.), StaticSimd::splat(WATER_H as f32))
        });
        vec![
            Layer {
                block: Block::Sand,
                height: Height::Constant(5.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::Ice,
                height: Height::OffsetNoise(i, (WATER_H as f32) - 1.),
                tag: LayerTag::Floating,
            },
        ]
    }

    fn generate_ocean(_generator: Grid<2>, _noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        vec![
            Layer {
                block: Block::Sand,
                height: Height::Constant(5.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::SeaBlock,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Liquid,
            },
        ]
    }

    fn generate_plain(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let plain_i = noise_samples.get_slot();
        let plain = &mut noise_samples[plain_i];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.08)
            .fill(plain);
        plain.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(5.),
                StaticSimd::splat(WATER_H as f32 + 15.),
            )
        });
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(plain_i),
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_mountain(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.08)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(30.),
                StaticSimd::splat(WATER_H as f32 + 15.),
            )
        });
        vec![Layer {
            block: Block::Granite,
            height: Height::Noise(i),
            tag: LayerTag::Mantle,
        }]
    }

    fn generate_desert(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let dunes_i = noise_samples.get_slot();
        let dunes = &mut noise_samples[dunes_i];
        generator
            .builder::<Billow, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.08)
            .fill(dunes);
        dunes.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(5.),
                StaticSimd::splat(WATER_H as f32 + 15.),
            )
        });
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Sand,
                height: Height::Noise(dunes_i),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_jungle(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.08)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(30.),
                StaticSimd::splat(WATER_H as f32 + 40.),
            )
        });
        vec![
            Layer {
                block: Block::Podzol,
                height: Height::Constant(WATER_H as f32 + 15.),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(i),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_canyon(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.08)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(-30.),
                StaticSimd::splat(WATER_H as f32 + 60.),
            )
        });
        vec![
            Layer {
                block: Block::CoarseDirt,
                height: Height::Noise(i),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Sand,
                height: Height::Constant(WATER_H as f32 + 8.),
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_tundra(generator: Grid<2>, noise_samples: &mut NoiseSamples) -> Vec<Layer> {
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Billow, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.08)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(20.),
                StaticSimd::splat(WATER_H as f32 + 20.),
            )
        });
        let icicles_i = noise_samples.get_slot();
        let icicles = &mut noise_samples[icicles_i];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 1.0)
            .fill(icicles);
        icicles.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(20.),
                StaticSimd::splat(WATER_H as f32 + 10.),
            )
        });
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Snow,
                height: Height::Noise(i),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::Ice,
                height: Height::Noise(icicles_i),
                tag: LayerTag::Deposit,
            },
        ]
    }
}

fn points_lerp_simd(a: StaticSimd<f32>, points: &[(f32, f32)]) -> StaticSimd<f32> {
    let (min_p, min_v) = points[0];
    let (max_p, max_v) = points[points.len() - 1];

    // Clamp to the boundary values first; everything else gets overwritten below for
    // whichever lanes actually land inside a segment.
    let below_mask = a.simd_lt(StaticSimd::splat(min_p));
    let above_mask = a.simd_ge(StaticSimd::splat(max_p));
    let mut result = below_mask.select(StaticSimd::splat(min_v), a);
    result = above_mask.select(StaticSimd::splat(max_v), result);

    for w in points.windows(2) {
        let (p1, v1) = w[0];
        let (p2, v2) = w[1];
        let slope = (v2 - v1) / (p2 - p1);
        let p1 = StaticSimd::splat(p1);
        let p2 = StaticSimd::splat(p2);
        let v1 = StaticSimd::splat(v1);
        let slope = StaticSimd::splat(slope);

        // p1 <= a && a < p2, mirroring the scalar version's segment check.
        let seg_mask = a.simd_ge(p1) & a.simd_lt(p2);
        let candidate = (a - p1).mul_add(slope, v1); // v1 + (a - p1) * slope
        result = seg_mask.select(candidate, result);
    }

    result
}
