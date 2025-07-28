use itertools::Itertools;
use riverbed_closest::{ClosestResult, ClosestTrait};
use riverbed_noise::*;
use crate::{generation::{biome_params::BiomeParameters, biomes::Biome, layer::LayerTag}, world::{unchunked, ColPos, VoxelWorld, CHUNK_S1}};

pub struct TerrainGenerator {
    pub biomes_points: Vec<([f32; 2], Biome)>,
    pub seed: u32,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let biomes_points = riverbed_closest::points::from_csv("assets/data/biomes.csv").unwrap();
        TerrainGenerator { seed, biomes_points }
    }

    pub fn generate(&self, world: &VoxelWorld, col: ColPos) {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let continentalness= fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed, 0.0001);
        let temperature = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed, 0.0005);
        let params = BiomeParameters {
            continentalness: continentalness,
            temperature: temperature,
        };
        let biomes_at_00 = self.biomes_points.closest(params.at(0, 0));
        let biomes_at_01 = self.biomes_points.closest(params.at(0, CHUNK_S1-1));
        let biomes_at_10 = self.biomes_points.closest(params.at(CHUNK_S1-1, 0));
        let biomes_at_11 = self.biomes_points.closest(params.at(CHUNK_S1-1, CHUNK_S1-1));
        // In the worst case scenario we will blend between 8 biomes, but in most cases it will be 2.
        let biomes: Vec<Biome> = [
            *biomes_at_00.closest,
            *biomes_at_00.next_closest.unwrap(),
            *biomes_at_01.closest,
            *biomes_at_01.next_closest.unwrap(),
            *biomes_at_10.closest,
            *biomes_at_10.next_closest.unwrap(),
            *biomes_at_11.closest,
            *biomes_at_11.next_closest.unwrap(),
        ].into_iter().unique().collect();
        let all_biome_layers = biomes.iter()
            .map(|b| b.generate(self.seed, col, &params))
            .collect::<Vec<_>>();
        // Blend between biomes
        let mut column_biome_weights = vec![0.0; biomes.len()];
        let mut layer_indexes = vec![0usize; biomes.len()];
        for dx in 0..CHUNK_S1 {
            for dz in 0..CHUNK_S1 {
                for (i, &biome) in biomes.iter().enumerate() {
                    column_biome_weights[i] = biome_weight(
                        &biomes_at_00, 
                        &biomes_at_01, 
                        &biomes_at_10, 
                        &biomes_at_11, 
                        dx, dz,
                        biome
                    );
                }
                layer_indexes.fill(0);
                let mut last_height = 0;
                while let Some(&min_layer_tag) = all_biome_layers.iter().zip(&layer_indexes).filter_map(|(layer, &i)| if i >= layer.len() { None } else { Some(&layer[i].tag) }).min() {
                    let mut n_min = 0.;
                    let mut h_min = 0.;
                    let mut h_other = 0.;
                    let mut dominant_block = None;
                    let mut max_weight = 0.;
                    for ((layer_idx, layers), &weight) in layer_indexes.iter_mut().zip(&all_biome_layers).zip(&column_biome_weights) {
                        if *layer_idx >= layers.len() || layers[*layer_idx].tag != min_layer_tag {
                            // This layer is above the min tag, we interpolate with the top of the preceeding layer
                            let target_height = if *layer_idx > 0 {
                                layers[*layer_idx-1].height(dx, dz)
                            } else { 0. };
                            h_other += target_height * weight;
                            continue;
                        }
                        h_min += layers[*layer_idx].height(dx, dz) * weight;
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
                    }.round() as i32;
                    if height <= last_height {
                        continue; // Don't overwrite lower layers
                    }
                    world.set_yrange(col, (dx, dz), height, (height-last_height) as usize, dominant_block.unwrap());               
                    last_height = height;
                }
            }
        }
    }
}

fn biome_weight(
    biomes_at_00: &ClosestResult<Biome>, 
    biomes_at_01: &ClosestResult<Biome>, 
    biomes_at_10: &ClosestResult<Biome>, 
    biomes_at_11: &ClosestResult<Biome>, 
    dx: usize, dz: usize,
    biome: Biome
) -> f32 {
    let x = (dx as f32) / (CHUNK_S1 - 1) as f32;
    let z = (dz as f32) / (CHUNK_S1 - 1) as f32;
    // Bilinear interpolation of the biome weights
    let x0_weight = (1.-x) * biomes_at_00.score(biome) +
        x * biomes_at_10.score(biome);
    let x1_weight = (1.-x) * biomes_at_01.score(biome) +
        x * biomes_at_11.score(biome);
    (1.-z) * x0_weight + z * x1_weight
}