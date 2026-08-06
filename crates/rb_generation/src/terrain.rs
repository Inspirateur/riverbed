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
use std::{collections::HashMap, ops::Div, path::Path};
const BIOME_SHARPENING: f32 = 100.;
const BIOME_EXCLUSION_THRESHOLD: f32 = 0.1;
pub(crate) const FREQ: f32 = 0.01;

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
    pub fn new(seed: u32, asset_path: &Path) -> Self {
        let biomes_points = BiomePoints::from_csv(asset_path.join("gen").join("biomes.csv"));
        let plant_ranges = PlantRanges::from_csv(asset_path.join("gen").join("plants.csv"));
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
        let simd_half = StaticSimd::splat(0.5);
        let continentalness = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.0005)
            .into_iter()
            .map(|v| v.mul_add(simd_half, simd_half))
            .collect();
        let mountainness = generator
            .builder::<Ridged, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.01)
            .into_iter()
            .map(|v| v * simd_half * v * simd_half)
            .collect();
        let temperature = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.0005)
            .into_iter()
            .map(|v| v.mul_add(simd_half, simd_half))
            .collect();
        let humidity = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.002)
            .into_iter()
            .map(|v| v.mul_add(simd_half, simd_half))
            .collect();
        let ph = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.005)
            .into_iter()
            .map(|v| v.mul_add(simd_half, simd_half))
            .collect();
        let trees = generator
            .builder::<Fbm, Perlin>()
            .octaves(3)
            .frequency(FREQ * 0.01)
            .into_iter()
            .map(|v| v.mul_add(simd_half, simd_half))
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

#[cfg(test)]
mod tests {
    use crate::TerrainGenerator;
    use quick_noise::Grid;
    use std::path::Path;
    const SIZE: usize = 8192;

    /// Checks that one a wide enough area the biome parameters are within [0, 1] and their mean is reasonable.
    #[test]
    fn check_noise_bound() {
        let test_generator = Grid::<2>::new(SIZE, SIZE);
        let terrain = TerrainGenerator::new(0, Path::new("../../assets"));
        let biome_params = terrain.biome_params_at(test_generator);
        for (param, values) in biome_params.0 {
            let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            // f64 because f32 can accumulate errors over a large number of samples
            let sum = values.iter().map(|&v| v as f64).sum::<f64>();
            let mean = sum / values.len() as f64;
            assert!(
                min >= 0. && max <= 1.,
                "Biome param {:?} out of bounds: [{}, {}] - mean: {}",
                param,
                min,
                max,
                mean
            );
            assert!(
                mean >= 0.35 && mean <= 0.65,
                "Biome param {:?} mean out of bounds: [{}, {}] - mean: {}",
                param,
                min,
                max,
                mean
            );
        }
    }
}
