use itertools::Itertools;
use riverbed_closest::{ClosestResult, ClosestTrait};
use riverbed_noise::*;
use crate::{generation::{biome_params::BiomeParameters, biomes::{Biome, Layer, LayerTag}}, world::{unchunked, ColPos, VoxelWorld, CHUNK_S1}, Block};

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
        for dx in 0..CHUNK_S1 {
            for dz in 0..CHUNK_S1 {
                let mut last_height = 0;
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
                let dominant_biome_index = column_biome_weights.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(i, _)| i)
                    .unwrap();
                for i in 0..all_biome_layers[dominant_biome_index].len() {
                    let block = all_biome_layers[dominant_biome_index][i].0;
                    let layer = &all_biome_layers[dominant_biome_index][i].1;
                    let tag = all_biome_layers[dominant_biome_index][i].2;
                    let weight = column_biome_weights[dominant_biome_index];
                    let height = if let LayerTag::Structure { base_height } = tag {
                        layer.height(dx, dz)*weight + base_height as f32 * (1. - weight)
                    } else if i < all_biome_layers[dominant_biome_index].len() - 1 
                        && let LayerTag::Structure { base_height } = all_biome_layers[dominant_biome_index][i + 1].2 
                    {
                        layer.height(dx, dz)*weight + base_height as f32 * (1. - weight)
                    } else {
                        (0..all_biome_layers.len())
                            .filter(|&i| i != dominant_biome_index)
                            .fold(
                                layer.height(dx, dz)*weight, 
                                |a, i| a + layer_height(&all_biome_layers[i], tag, dx, dz)*column_biome_weights[i]
                            )
                    } as i32;
                    if height < last_height {
                        continue; // Don't overwrite lower layers
                    }
                    world.set_yrange(col, (dx, dz), height, (height-last_height) as usize, block);               
                    last_height = height;
                }
            }
        }
    }
}

fn layer_height(column: &Vec<(Block, Layer, LayerTag)>, layer: LayerTag, dx: usize, dz: usize) -> f32 {
    column.iter()
        .find(|(_, _, tag)| *tag == layer)
        .map(|(_, layer, _)| layer.height(dx, dz))
        .unwrap_or(0.0)
}

fn biome_weight(
    biome_at_00: &ClosestResult<Biome>, 
    biome_at_01: &ClosestResult<Biome>, 
    biome_at_10: &ClosestResult<Biome>, 
    biome_at_11: &ClosestResult<Biome>, 
    dx: usize, dz: usize,
    biome: Biome
) -> f32 {
    let x = (dx as f32) / (CHUNK_S1 - 1) as f32;
    let z = (dz as f32) / (CHUNK_S1 - 1) as f32;
    // Bilinear interpolation of the biome weights
    let x0_weight = (1.-x) * biome_at_00.score(biome) +
        x * biome_at_10.score(biome);
    let x1_weight = (1.-x) * biome_at_01.score(biome) +
        x * biome_at_11.score(biome);
    (1.-z) * x0_weight + z * x1_weight
}