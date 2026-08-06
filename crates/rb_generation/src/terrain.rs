use crate::{
    biome_params::*,
    biomes::Biome,
    coverage::CoverageTrait,
    layer::{Layer, LayerTag},
    noise_samples::NoiseSamples,
    plant_params::PlantRanges,
    tree::TreeSeed,
};
use bevy::{ecs::system::IntoResult, log::info_span};
use quick_noise::{
    Fbm, Grid, Perlin, Ridged,
    api::batch::interface::DimTuple,
    simd::{SimdSliceIterExt, StaticSimd},
};
use rb_block::Block;
use rb_world::{
    BlockPos2d, CHUNK_S1, ChunkPos2d, ChunkedPos2d, Column, MAX_GEN_HEIGHT, StructureTrait,
};
use std::{collections::HashMap, ops::Div};
const BIOME_SHARPENING: f32 = 100.;
const BIOME_EXCLUSION_THRESHOLD: f32 = 0.1;
pub(crate) const FREQ: f32 = 0.1;

pub struct TerrainGenerator {
    pub biomes_points: BiomePoints<4>,
    pub plant_ranges: PlantRanges<4>,
    pub seed: u32,
    generator: Grid<2>,
    column_biome_weights: Box<[f32]>,
    layer_indexes: Box<[usize]>,
    noise_samples: NoiseSamples,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let biomes_points = BiomePoints::from_csv("assets/gen/biomes.csv");
        let plant_ranges = PlantRanges::from_csv("assets/gen/plants.csv");
        TerrainGenerator {
            seed,
            column_biome_weights: vec![0.0; biomes_points.points.len()].into_boxed_slice(),
            layer_indexes: vec![0usize; biomes_points.points.len()].into_boxed_slice(),
            biomes_points,
            plant_ranges,
            generator: Grid::<2>::new(CHUNK_S1, CHUNK_S1).seed(seed as i64),
            noise_samples: NoiseSamples::new(),
        }
    }

    fn clear(&mut self) {
        self.column_biome_weights.fill(0.);
        self.layer_indexes.fill(0);
        self.noise_samples.clear();
    }

    /// Return normalized per 2d block biome weights given the biome parameters and a threshold for considering a biome relevant.
    /// The biome scores are computed from their distance to the biome parameters, then sharpened and normalized.
    /// Biomes with scores that are all under BIOME_EXCLUSION_THRESHOLD (before normalization) will be excluded from the result.
    /// Will panic instead of returning 0 biomes.
    fn biome_scores(&self, params: BiomeParameters) -> Vec<(Biome, [f32; CHUNK_S1 * CHUNK_S1])> {
        let simd_threshold = StaticSimd::splat(BIOME_EXCLUSION_THRESHOLD);
        let simd_one = StaticSimd::splat(1.);
        let mut res = Vec::new();
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
                // To sharpen them we take the inverse of the square distance (by omitting .sqrt)
                .for_each(|mut weight| *weight = simd_one.div(*weight));

            let mut max = StaticSimd::splat(0.);
            biome_weights
                .simd_iter()
                .for_each(|weight| max = max.max(weight));

            if max.simd_gt(simd_threshold).all_false() {
                continue;
            }
            // Normalize the weights
            // We use SIMD to sum the weights, then divide each weight by the sum
            let sum = biome_weights
                .simd_iter()
                .fold(StaticSimd::splat(0.), |acc, weight| acc + weight)
                .to_array()
                .into_iter()
                .sum::<f32>();
            let simd_sum = StaticSimd::splat(sum);
            biome_weights
                .simd_iter_mut()
                .for_each(|mut weight| *weight = *weight / simd_sum);
            res.push((biome.clone(), biome_weights));
        }
        assert!(
            !res.is_empty(),
            "Returning 0 biomes check your biome parameters and the BIOME_EXCLUSION_THRESHOLD constant"
        );
        res
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
        let biomes: Vec<Biome> = self
            .biomes_points
            .closest_biomes(params.average(self.biomes_points.parameters), 1.);
        let all_biome_layers = biomes
            .iter()
            .map(|b| b.generate(generator, &mut self.noise_samples))
            .collect::<Vec<_>>();
        biome_gen_span.exit();
        let biome_param_span = info_span!("terrain", name = "biome param gather").entered();
        let param_points = params.view(self.biomes_points.parameters);
        biome_param_span.exit();
        let blending_span = info_span!("terrain", name = "biome blending").entered();
        // Blend between biomes
        for dx in 0..CHUNK_S1 {
            for dz in 0..CHUNK_S1 {
                // Compute normalized biome weights for this block column
                if biomes.len() > 1 {
                    let biome_params = param_points[dx + dz * CHUNK_S1];
                    let mut total = 0.;
                    for (i, &biome) in biomes.iter().enumerate() {
                        self.column_biome_weights[i] =
                            (-self.biomes_points.dist_from(&biome_params, &biome)
                                * BIOME_SHARPENING)
                                .exp();
                        total += self.column_biome_weights[i];
                    }
                    for i in 0..self.column_biome_weights.len() {
                        self.column_biome_weights[i] = self.column_biome_weights[i] / total;
                    }
                } else {
                    self.column_biome_weights[0] = 1.
                }
                // Blend biome layers
                self.layer_indexes.fill(0);
                let mut last_height = 0;
                while let Some(&min_layer_tag) = all_biome_layers
                    .iter()
                    .zip(&self.layer_indexes)
                    .filter_map(|(layer, &i)| {
                        if i >= layer.len() {
                            None
                        } else {
                            Some(&layer[i].tag)
                        }
                    })
                    .min()
                {
                    let mut n_min = 0.;
                    let mut h_min = 0.;
                    let mut h_other = 0.;
                    let mut dominant_block = None;
                    let mut max_weight = 0.;
                    for ((layer_idx, layers), &weight) in self
                        .layer_indexes
                        .iter_mut()
                        .zip(&all_biome_layers)
                        .zip(&self.column_biome_weights)
                    {
                        if *layer_idx >= layers.len() || layers[*layer_idx].tag != min_layer_tag {
                            // This layer is above the min tag, we interpolate with the top of the preceeding layer
                            let target_height = if *layer_idx > 0 {
                                layers[*layer_idx - 1].height(&self.noise_samples, dx, dz)
                            } else {
                                0.
                            };
                            h_other += target_height * weight;
                            continue;
                        }
                        h_min += layers[*layer_idx].height(&self.noise_samples, dx, dz) * weight;
                        n_min += weight;
                        if weight > max_weight {
                            max_weight = weight;
                            dominant_block = Some(layers[*layer_idx].block);
                        }
                        *layer_idx += 1;
                    }
                    // We shouldn't need this but floats accumulate errors
                    n_min = n_min.clamp(0., 1.);
                    h_min /= n_min;
                    let n_other = 1. - n_min;
                    if n_other > 0. {
                        h_other /= n_other;
                    }
                    if let LayerTag::Fixed { height } = min_layer_tag {
                        h_other = height as f32;
                    }
                    let height = if h_min < h_other {
                        h_min
                    } else {
                        h_min * n_min + h_other * n_other
                    }
                    .round() as i32;
                    if height < last_height {
                        continue; // Don't overwrite lower layers
                    }
                    let block = dominant_block.unwrap();
                    let layer_width = (height - last_height).max(1);
                    if block == Block::GrassBlock {
                        column.set_yrange(ChunkedPos2d { x: dx, z: dz }, height, 1, block);
                        if layer_width > 1 {
                            column.set_yrange(
                                ChunkedPos2d { x: dx, z: dz },
                                height - 1,
                                (layer_width - 1) as usize,
                                Block::Dirt,
                            );
                        }
                    } else {
                        column.set_yrange(
                            ChunkedPos2d { x: dx, z: dz },
                            height,
                            layer_width as usize,
                            block,
                        );
                    }
                    last_height = height;
                }
            }
        }
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
        let continentalness = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.0005)
            .into_iter()
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
            .collect();
        let humidity = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.002)
            .into_iter()
            .collect();
        let ph = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.005)
            .into_iter()
            .collect();
        let trees = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.01)
            .into_iter()
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
