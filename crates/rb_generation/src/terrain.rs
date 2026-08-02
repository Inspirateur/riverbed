use crate::{
    biome_params::*,
    biomes::Biome,
    coverage::CoverageTrait,
    layer::{Height, Layer, LayerTag},
    noise_samples::NoiseSamples,
    plant_params::PlantRanges,
    tree::TreeSeed,
};
use bevy::log::info_span;
use quick_noise::{
    Fbm, Grid, Perlin, Ridged,
    simd::{SimdSliceIterExt, StaticSimd},
};
use rb_block::Block;
use rb_world::{
    BlockPos2d, CHUNK_S1, ChunkPos2d, ChunkedPos2d, Column, MAX_GEN_HEIGHT, StructureTrait,
};
use std::{collections::HashMap, ops::Div};
use strum::IntoEnumIterator;
const BIOME_SHARPENING: f32 = 100.;
const BIOME_EXCLUSION_THRESHOLD: f32 = 0.;
pub(crate) const FREQ: f32 = 0.1;

pub struct TerrainGenerator {
    pub biomes_points: BiomePoints<4>,
    pub plant_ranges: PlantRanges<4>,
    pub seed: u32,
    generator: Grid<2>,
    layer_indexes: Box<[usize]>,
    noise_samples: NoiseSamples,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let biomes_points = BiomePoints::from_csv("assets/gen/biomes.csv");
        let plant_ranges = PlantRanges::from_csv("assets/gen/plants.csv");
        TerrainGenerator {
            seed,
            layer_indexes: vec![0usize; biomes_points.points.len()].into_boxed_slice(),
            biomes_points,
            plant_ranges,
            generator: Grid::<2>::new(CHUNK_S1, CHUNK_S1).seed(seed as i64),
            noise_samples: NoiseSamples::new(),
        }
    }

    fn clear(&mut self) {
        self.layer_indexes.fill(0);
        self.noise_samples.clear();
    }

    /// Return normalized per 2d block biome weights given the biome parameters and a threshold for considering a biome relevant.
    /// The biome scores are computed from their distance to the biome parameters, then sharpened and normalized.
    /// Biomes with scores that are all under BIOME_EXCLUSION_THRESHOLD (before normalization) will be excluded from the result.
    /// Will panic instead of returning 0 biomes.
    fn biome_scores(&self, params: &BiomeParameters) -> HashMap<Biome, [f32; CHUNK_S1 * CHUNK_S1]> {
        let simd_threshold = StaticSimd::splat(BIOME_EXCLUSION_THRESHOLD);
        let simd_one = StaticSimd::splat(1.);
        let mut res = HashMap::new();
        for (point, biome) in &self.biomes_points.points {
            let mut biome_weights = [0.0; CHUNK_S1 * CHUNK_S1];
            for (param, value) in self.biomes_points.parameters.iter().zip(point) {
                let simd_value = StaticSimd::splat(*value);
                biome_weights
                    .simd_iter_mut()
                    .zip(params[*param].simd_iter())
                    .for_each(|(mut weight, param_value)| {
                        *weight += (param_value - simd_value) * (param_value - simd_value);
                    });
            }
            biome_weights
                .simd_iter_mut()
                // To sharpen them we do 1 / (1 + distance)^4
                // (because we omit the sqrt)
                .for_each(|mut weight| {
                    *weight = simd_one.div((*weight + simd_one) * (*weight + simd_one))
                });

            let mut max = StaticSimd::splat(0.);
            biome_weights
                .simd_iter()
                .for_each(|weight| max = max.max(weight));

            // With current parameters, biomes with a distance >~0.78 in parameter space are excluded
            if max.simd_gt(simd_threshold).all_false() {
                continue;
            }
            res.insert(biome.clone(), biome_weights);
        }
        assert!(
            !res.is_empty(),
            "Returning 0 biomes check your biome parameters and the BIOME_EXCLUSION_THRESHOLD constant"
        );
        // Normalize the weights
        let mut sum = [0.0; CHUNK_S1 * CHUNK_S1];
        for weights in res.values() {
            sum.simd_iter_mut_static()
                .zip(weights.simd_iter())
                .for_each(|(mut s, weight)| *s += weight);
        }
        for weights in res.values_mut() {
            weights
                .simd_iter_mut_static()
                .zip(sum.simd_iter())
                .for_each(|(mut weight, s)| *weight /= s);
        }
        res
    }

    /// Zips the biome layers by layer tag for blending.
    ///
    /// For interpolated layers only:
    ///   if a layer tag is missing for a biome, the layer will be substituted by the previous one,
    ///   if none exist, it will be a constant 0.
    fn zip_layers(
        all_biome_layers: Vec<(Biome, Vec<Layer>)>,
    ) -> Vec<(LayerTag, Vec<(Biome, Layer)>)> {
        LayerTag::iter()
            .map(|tag| {
                if tag.should_interpolate() {
                    (
                        tag,
                        all_biome_layers
                            .iter()
                            .map(|(biome, layers)| {
                                (
                                    *biome,
                                    layers
                                        .iter()
                                        .rev()
                                        .find(|l| l.tag <= tag && l.tag.should_interpolate())
                                        .unwrap_or(&Layer {
                                            block: Block::Air,
                                            height: Height::Constant(0.),
                                            tag,
                                        })
                                        .clone(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    (
                        tag,
                        all_biome_layers
                            .iter()
                            .flat_map(|(biome, layers)| {
                                layers
                                    .iter()
                                    .find(|l| l.tag == tag)
                                    .cloned()
                                    .map(|l| (*biome, l))
                            })
                            .collect::<Vec<_>>(),
                    )
                }
            })
            .collect()
    }

    pub fn generate_with_params(
        &mut self,
        generator: Grid<2>,
        col: ChunkPos2d,
        params: BiomeParameters,
    ) -> (Column, Vec<Box<dyn StructureTrait>>) {
        let column_gen_span = info_span!("terrain", name = "column generation").entered();
        self.clear();
        let mut column = Column::default();
        let biome_gen_span = info_span!("terrain", name = "biome layer gen").entered();
        let mut structures: Vec<Box<dyn StructureTrait>> = Vec::new();
        // The biomes that will be considered for blending in this chunk
        let biome_weights = self.biome_scores(&params);

        let mut max_weights = [0f32; CHUNK_S1 * CHUNK_S1];
        biome_weights.values().for_each(|weights| {
            max_weights
                .simd_iter_mut()
                .zip(weights.simd_iter())
                .for_each(|(mut max, weight): (_, StaticSimd<f32>)| *max = (*max).max(weight));
        });
        let all_biome_layers = Self::zip_layers(
            biome_weights
                .iter()
                .map(|(b, _)| (*b, b.generate(generator, &mut self.noise_samples)))
                .collect::<Vec<_>>(),
        );
        biome_gen_span.exit();
        let blending_span = info_span!("terrain", name = "biome blending").entered();
        let mut blend_result = [0.0; CHUNK_S1 * CHUNK_S1];
        let mut previous_layer_heights = [0; CHUNK_S1 * CHUNK_S1];
        for (layer_tag, layers) in all_biome_layers {
            if layer_tag.should_interpolate() {
                blend_result.fill(0.);
                for (biome, layer) in &layers {
                    let biome_weight = biome_weights.get(biome).unwrap();
                    match layer.height {
                        Height::Constant(h) => {
                            let simd_h = StaticSimd::splat(h);
                            blend_result
                                .simd_iter_mut()
                                .zip(biome_weight.simd_iter())
                                .for_each(|(mut res, weight)| *res += simd_h * weight);
                        }
                        Height::Noise(i) => {
                            blend_result
                                .simd_iter_mut_static()
                                .zip(biome_weight.simd_iter())
                                .zip(self.noise_samples[i].simd_iter())
                                .for_each(|((mut res, weight), sample)| *res += sample * weight);
                        }
                        Height::OffsetNoise(i, h) => {
                            let simd_h = StaticSimd::splat(h);
                            blend_result
                                .simd_iter_mut()
                                .zip(biome_weight.simd_iter())
                                .zip(self.noise_samples[i].simd_iter())
                                .for_each(|((mut res, weight), sample)| {
                                    *res += (sample + simd_h) * weight
                                });
                        }
                    };
                }
                for (biome, layer) in &layers {
                    // We can reuse previous layers for interpolation but not for placing blocks
                    if layer.tag != layer_tag {
                        continue;
                    }
                    let biome_weight = biome_weights.get(biome).unwrap();
                    for dx in 0..CHUNK_S1 {
                        for dz in 0..CHUNK_S1 {
                            let i = dx + dz * CHUNK_S1;
                            // only place block if this biome is the dominant one for this block column
                            if biome_weight[i] < max_weights[i] {
                                continue;
                            }
                            let top = blend_result[i].round() as i32;
                            let bottom = if let Height::OffsetNoise(_, h) = layer.height {
                                h as i32
                            } else {
                                previous_layer_heights[i]
                            };
                            let height = (top - bottom + 1) as usize;
                            if height <= 0 {
                                continue;
                            }
                            previous_layer_heights[i] = top;
                            column.set_yrange(dx, dz, top, height, layer.block);
                        }
                    }
                }
            } else {
                for (_, fixed_layer) in &layers {
                    for dx in 0..CHUNK_S1 {
                        for dz in 0..CHUNK_S1 {
                            let height =
                                fixed_layer.height(&self.noise_samples, dx, dz).round() as i32;
                            column.set_yrange(dx, dz, height, 1, fixed_layer.block);
                        }
                    }
                }
            }
        }
        // Blend between biomes
        blending_span.exit();
        let structure_gen_span = info_span!("terrain", name = "structure generation").entered();
        let tree_spots = [
            (0, 0),
            (15, 0),
            (31, 0),
            (46, 0),
            (8, 15),
            (24, 15),
            (40, 15),
            (0, 31),
            (15, 31),
            (31, 31),
            (46, 31),
            (8, 46),
            (24, 46),
            (40, 46),
        ];
        for spot in tree_spots {
            let rng = <BlockPos2d>::from((
                col,
                ChunkedPos2d {
                    x: spot.0,
                    z: spot.1,
                },
            ))
            .prng(self.seed as i32);
            let dx = spot.0 + (rng & 0b111);
            let dz = spot.1 + ((rng >> 3) & 0b111);
            let i = dx * CHUNK_S1 + dz;
            let tree = params[BiomeParam::Trees][i];
            if tree < 0.2 {
                continue;
            }
            let h = (rng >> 6) & 0b11;
            let (block, y) = column.top_block(ChunkedPos2d { x: dx, z: dz });
            if !block.is_fertile_soil() {
                continue;
            }
            let (tree, dist) = self.plant_ranges.closest([
                params[BiomeParam::Temperature][i],
                params[BiomeParam::Humidity][i],
                params[BiomeParam::Ph][i],
                y as f32 / MAX_GEN_HEIGHT as f32,
            ]);
            if dist >= 0. {
                let pos = (col, (dx, y, dz)).into();
                structures.push(Box::new(TreeSeed {
                    pos,
                    species: tree.clone(),
                    size: dist + h as f32 / 10.,
                }));
            }
        }
        structure_gen_span.exit();
        column_gen_span.exit();
        (column, structures)
    }

    pub fn biome_params_at(&self, generator: Grid<2>) -> BiomeParameters {
        let simd_half = StaticSimd::splat(0.5);
        let continentalness = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.0005)
            .into_iter()
            .map(|s| s.mul_add(simd_half, simd_half))
            .collect();
        let mountainness = generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.001)
            .into_iter()
            .collect();
        let temperature = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.0005)
            .into_iter()
            .map(|s| s.mul_add(simd_half, simd_half))
            .collect();
        let humidity = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.002)
            .into_iter()
            .map(|s| s.mul_add(simd_half, simd_half))
            .collect();
        let ph = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.005)
            .into_iter()
            .map(|s| s.mul_add(simd_half, simd_half))
            .collect();
        let trees = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.01)
            .into_iter()
            .map(|s| s.mul_add(simd_half, simd_half))
            .collect();
        BiomeParameters(HashMap::from([
            (BiomeParam::Continentalness, continentalness),
            (BiomeParam::Mountainness, mountainness),
            (BiomeParam::Temperature, temperature),
            (BiomeParam::Humidity, humidity),
            (BiomeParam::Ph, ph),
            (BiomeParam::Trees, trees),
        ]))
    }

    pub fn generate(&mut self, col: ChunkPos2d) -> (Column, Vec<Box<dyn StructureTrait>>) {
        let generator = self.generator.grid_position(col.x, col.z);
        let params = self.biome_params_at(generator);
        self.generate_with_params(generator, col, params)
    }
}
