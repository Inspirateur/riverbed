use crate::{
    biome_params::{BiomeParam, BiomeParameters},
    layer::*,
    noise_samples::NoiseSamples,
    terrain::FREQ,
};
use quick_noise::{Billow, Fbm, Grid, Perlin, Ridged, simd::SimdSliceIterExt, simd::StaticSimd};
use rb_block::Block;
use rb_world::{CHUNK_S1, WATER_H};
use std::ops::Mul;
use strum_macros::EnumString;
const MOUNTAIN_H: f32 = 150.;
const MOUNTAIN_GRASS_H: f32 = 2.;
const MOUNTAIN_GRASS_MAX_SLOPE: f32 = 0.8;

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
    JunglePillars,
}

impl Biome {
    pub fn generate(
        &self,
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        params: &BiomeParameters,
    ) -> Vec<Layer> {
        match self {
            Biome::PolarOcean => Biome::generate_polar_ocean(generator, noise_samples, params),
            Biome::Ocean => Biome::generate_ocean(generator, noise_samples, params),
            Biome::Mountain => Biome::generate_mountain(generator, noise_samples, params),
            Biome::Desert => Biome::generate_desert(generator, noise_samples, params),
            Biome::Jungle => Biome::generate_jungle(generator, noise_samples, params),
            Biome::Canyon => Biome::generate_canyon(generator, noise_samples, params),
            Biome::Tundra => Biome::generate_tundra(generator, noise_samples, params),
            Biome::JunglePillars => {
                Biome::generate_jungle_pillars(generator, noise_samples, params)
            }
            // Plain is the default for any not yet implemented biomes
            _ => Biome::generate_plain(generator, noise_samples, params),
        }
    }

    fn generate_polar_ocean(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
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
                block: Block::SeaBlock,
                height: Height::Constant(WATER_H as f32),
                tag: LayerTag::Fixed {
                    height: WATER_H as usize,
                },
            },
            Layer {
                block: Block::Ice,
                height: Height::Noise(i),
                tag: LayerTag::Fixed {
                    height: (WATER_H as usize) - 1,
                },
            },
        ]
    }

    fn generate_ocean(
        _generator: Grid<2>,
        _noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
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

    fn generate_plain(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        params: &BiomeParameters,
    ) -> Vec<Layer> {
        let plain_i = noise_samples.get_slot();
        let plain = &mut noise_samples[plain_i];
        let simd_half = StaticSimd::splat(0.5);
        generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.6)
            .fill(plain);
        plain
            .simd_iter_mut_static()
            .zip(params.0[&BiomeParam::Mountainness].simd_iter_static())
            .for_each(|(mut v, m)| {
                *v = v.mul_add(simd_half, simd_half);
                *v = v.mul_add(
                    StaticSimd::splat(300.) * m * m * m * m,
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
                block: Block::GrassBlock,
                height: Height::Noise(plain_i),
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_mountain(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
        let simd_half = StaticSimd::splat(0.5);
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(4)
            .frequency(FREQ * 0.3)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(simd_half, simd_half);
            *v = v.mul_add(
                StaticSimd::splat(250.),
                StaticSimd::splat(WATER_H as f32 + 5.),
            )
        });
        let grass_i = noise_samples.get_slot();
        // The grass slot first holds the slope, then the grass top height
        let (mountain, grass) = noise_samples.pair_mut(i, grass_i);
        slope_simd(mountain, grass);
        // Grass thickness goes from MOUNTAIN_GRASS_H on flat ground to 0 at MOUNTAIN_GRASS_MAX_SLOPE
        let grass_h = StaticSimd::splat(MOUNTAIN_GRASS_H);
        let grass_falloff = StaticSimd::splat(-MOUNTAIN_GRASS_H / MOUNTAIN_GRASS_MAX_SLOPE);
        grass
            .simd_iter_mut_static()
            .zip(mountain.simd_iter_static())
            .for_each(|(mut s, m)| {
                *s = m + s.mul_add(grass_falloff, grass_h).max(StaticSimd::splat(0.));
            });
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Noise(i),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(grass_i),
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_desert(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
        let dunes_i = noise_samples.get_slot();
        let dunes = &mut noise_samples[dunes_i];
        generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.2)
            .fill(dunes);
        dunes.simd_iter_mut().for_each(|mut v| {
            *v = v.sqrt().mul_add(
                StaticSimd::splat(20.),
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
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_jungle(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
        const TERRACE_H: f32 = 3.;
        let step = StaticSimd::splat(TERRACE_H);
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.2)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(30.),
                StaticSimd::splat(WATER_H as f32 + 20.),
            );
            *v = (*v / step).round() * step;
        });
        vec![
            Layer {
                block: Block::GrassBlock,
                height: Height::Noise(i),
                tag: LayerTag::Soil,
            },
            Layer {
                block: Block::Podzol,
                height: Height::Constant(WATER_H as f32 + 16.),
                tag: LayerTag::Deposit,
            },
        ]
    }

    fn generate_canyon(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
        let simd_half = StaticSimd::splat(0.5);
        let simd_one = StaticSimd::splat(1.0);
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        let points = [
            (
                StaticSimd::splat(0.),
                StaticSimd::splat(WATER_H as f32 + 20.),
            ),
            (
                StaticSimd::splat(0.4),
                StaticSimd::splat(WATER_H as f32 + 25.),
            ),
            (
                StaticSimd::splat(0.45),
                StaticSimd::splat(WATER_H as f32 + 70.),
            ),
            (
                StaticSimd::splat(1.0),
                StaticSimd::splat(WATER_H as f32 + 73.),
            ),
        ];
        generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.2)
            .fill(n);
        n.simd_iter_mut_static()
            .for_each(|mut v| *v = points_lerp_simd(v.mul_add(-simd_half, simd_one), &points));
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

    fn generate_jungle_pillars(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        _params: &BiomeParameters,
    ) -> Vec<Layer> {
        let simd_half = StaticSimd::splat(0.5);
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        let points = [
            (StaticSimd::splat(0.), StaticSimd::splat(WATER_H as f32)),
            (StaticSimd::splat(0.6), StaticSimd::splat(WATER_H as f32)),
            (
                StaticSimd::splat(0.65),
                StaticSimd::splat(WATER_H as f32 + 70.),
            ),
            (
                StaticSimd::splat(1.0),
                StaticSimd::splat(WATER_H as f32 + 80.),
            ),
        ];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.2)
            .fill(n);
        n.simd_iter_mut_static()
            .for_each(|mut v| *v = points_lerp_simd(v.mul_add(simd_half, simd_half), &points));
        vec![
            Layer {
                block: Block::Granite,
                height: Height::Noise(i),
                tag: LayerTag::Mantle,
            },
            Layer {
                block: Block::Podzol,
                height: Height::Constant(WATER_H as f32 + 20.),
                tag: LayerTag::Soil,
            },
        ]
    }

    fn generate_tundra(
        generator: Grid<2>,
        noise_samples: &mut NoiseSamples,
        params: &BiomeParameters,
    ) -> Vec<Layer> {
        const ICICLES_MAX_TEMP: f32 = 0.4;
        const ICICLES_MIN_CONT: f32 = 0.55;
        const ICICLES_OFFSET: f32 = 0.3;
        let icicles_max_temp_simd = StaticSimd::splat(ICICLES_MAX_TEMP);
        let icicles_min_cont_simd = StaticSimd::splat(ICICLES_MIN_CONT);
        let icicles_offset_simd = StaticSimd::splat(ICICLES_OFFSET);
        let simd_zero = StaticSimd::splat(0.);
        let simd_icicles_height = StaticSimd::splat(140.);
        let i = noise_samples.get_slot();
        let n = &mut noise_samples[i];
        generator
            .builder::<Billow, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.1)
            .fill(n);
        n.simd_iter_mut().for_each(|mut v| {
            *v = v.mul_add(
                StaticSimd::splat(10.),
                StaticSimd::splat(WATER_H as f32 + 20.),
            )
        });
        let icicles_i = noise_samples.get_slot();
        let icicles = &mut noise_samples[icicles_i];
        generator
            .builder::<Fbm, Perlin>()
            .octaves(2)
            .frequency(FREQ * 0.5)
            .fill(icicles);
        icicles
            .simd_iter_mut_static()
            .zip(params.0[&BiomeParam::Continentalness].simd_iter_static())
            .zip(params.0[&BiomeParam::Temperature].simd_iter_static())
            .for_each(|((mut v, c), t)| {
                *v = (c.simd_gt(icicles_min_cont_simd) & t.simd_lt(icicles_max_temp_simd)).select(
                    (*v - icicles_offset_simd)
                        .mul_add(simd_icicles_height, StaticSimd::splat(WATER_H as f32)),
                    simd_zero,
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

fn points_lerp_simd(
    a: StaticSimd<f32>,
    points: &[(StaticSimd<f32>, StaticSimd<f32>)],
) -> StaticSimd<f32> {
    let (min_p, min_v) = points[0];
    let (max_p, max_v) = points[points.len() - 1];

    // Clamp to the boundary values first; everything else gets overwritten below for
    // whichever lanes actually land inside a segment.
    let below_mask = a.simd_lt(min_p);
    let above_mask = a.simd_ge(max_p);
    let mut result = below_mask.select(min_v, a);
    result = above_mask.select(max_v, result);

    for w in points.windows(2) {
        let (p1, v1) = w[0];
        let (p2, v2) = w[1];
        let slope = (v2 - v1) / (p2 - p1);

        // p1 <= a && a < p2, mirroring the scalar version's segment check.
        let seg_mask = a.simd_ge(p1) & a.simd_lt(p2);
        let candidate = (a - p1).mul_add(slope, v1); // v1 + (a - p1) * slope
        result = seg_mask.select(candidate, result);
    }

    result
}

/// Slope (gradient magnitude, in height units per block) of a `CHUNK_S1 x CHUNK_S1` height map
/// laid out as `x + z * CHUNK_S1`, using central finite differences on the 4 neighbors.
/// Border cells use clamped neighbors (one-sided difference).
pub fn slope_simd(h: &[f32; CHUNK_S1 * CHUNK_S1], out: &mut [f32; CHUNK_S1 * CHUNK_S1]) {
    const S: usize = CHUNK_S1;
    let lanes = StaticSimd::<f32>::LANES;
    let half = StaticSimd::splat(0.5);

    for z in 1..S - 1 {
        let row = z * S;
        for x in (1..S - 1).step_by(lanes) {
            let len = lanes.min(S - 1 - x);
            let i = row + x;
            let load = |start: usize| StaticSimd::<f32>::from_slice(&h[start..start + len]);
            let dx = (load(i + 1) - load(i - 1)) * half;
            let dz = (load(i + S) - load(i - S)) * half;
            dx.mul_add(dx, dz * dz)
                .sqrt()
                .copy_to_slice(&mut out[i..i + len]);
        }
    }

    // Border cells (scalar, clamped neighbors)
    let at = |x: usize, z: usize| h[x + z * S];
    for z in 0..S {
        let step = if z == 0 || z == S - 1 { 1 } else { S - 1 };
        for x in (0..S).step_by(step) {
            let (x0, x1) = (x.saturating_sub(1), (x + 1).min(S - 1));
            let (z0, z1) = (z.saturating_sub(1), (z + 1).min(S - 1));
            let dx = (at(x1, z) - at(x0, z)) / (x1 - x0) as f32;
            let dz = (at(x, z1) - at(x, z0)) / (z1 - z0) as f32;
            out[x + z * S] = dx.hypot(dz);
        }
    }
}
