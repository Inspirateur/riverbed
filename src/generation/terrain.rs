use riverbed_noise::*;
use crate::{generation::{biome_params::{BiomeParameters, BiomePoints}, biomes::Biome, layer::LayerTag}, world::{unchunked, ColPos, VoxelWorld, CHUNK_S1}};
const BIOME_SHARPENING: f32 = 50.;

pub struct TerrainGenerator {
    pub biomes_points: BiomePoints<3>,
    pub seed: u32,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let biomes_points = BiomePoints::from_csv("assets/gen/biomes.csv");
        TerrainGenerator { seed, biomes_points }
    }

    pub fn generate(&self, world: &VoxelWorld, col: ColPos) {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let continentalness= fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed, 0.0005);
        let temperature = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+1, 0.001);
        let mountain_veins = ridge(x, CHUNK_S1, z, CHUNK_S1, self.seed+2, 0.005);
        let mountain_presence = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+3, 0.005);
        let mountainness = mountain_presence.into_iter().zip(mountain_veins).map(|(p, v)| p*v).collect();
        let params = BiomeParameters {
            continentalness, temperature, mountainness
        };
        // The biomes that will be considered for blending in this chunk
        let biomes: Vec<Biome> = self.biomes_points.closest_biomes(params.at(CHUNK_S1/2, CHUNK_S1/2), 1.);
        let all_biome_layers = biomes.iter()
            .map(|b| b.generate(self.seed, col, &params))
            .collect::<Vec<_>>();
        // Blend between biomes
        let mut column_biome_weights = vec![0.0; biomes.len()];
        let mut layer_indexes = vec![0usize; biomes.len()];
        for dx in 0..CHUNK_S1 {
            for dz in 0..CHUNK_S1 {
                // Compute normalized biome weights for this block column
                if biomes.len() > 1 {
                    let biome_params = params.at(dx, dz);
                    let mut total = 0.;
                    for (i, &biome) in biomes.iter().enumerate() {
                        column_biome_weights[i] = (-self.biomes_points.dist_from(&biome_params, &biome)*BIOME_SHARPENING).exp();
                        total += column_biome_weights[i];
                    }
                    for i in 0..column_biome_weights.len() {
                        column_biome_weights[i] = column_biome_weights[i]/total;
                    }
                } else {
                    column_biome_weights[0] = 1.
                }
                // Blend biome layers
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