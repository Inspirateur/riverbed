use riverbed_noise::*;
use crate::{generation::{biome_params::{BiomeParameters, BiomePoints}, biomes::Biome, coverage::CoverageTrait, layer::LayerTag, plant_params::PlantRanges}, world::{unchunked, BlockPos, BlockPos2d, ColPos, VoxelWorld, CHUNK_S1, CHUNK_S1I, MAX_GEN_HEIGHT}, Block};
const BIOME_SHARPENING: f32 = 50.;

pub struct TerrainGenerator {
    pub biomes_points: BiomePoints<4>,
    pub plant_ranges: PlantRanges<4>,
    pub seed: u32,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let biomes_points = BiomePoints::from_csv("assets/gen/biomes.csv");
        let plant_ranges = PlantRanges::from_csv("assets/gen/plants.csv");
        TerrainGenerator { seed, biomes_points, plant_ranges }
    }

    pub fn generate(&self, world: &VoxelWorld, col: ColPos) {
        let (x, z) = (unchunked(col.x, 0) as f32, unchunked(col.z, 0) as f32);
        let continentalness= fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed, 0.0005);
        let mountainness = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+1, 0.005);
        let temperature = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+2, 0.002);
        let humidity = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+3, 0.002);
        let trees = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+4, 0.005);
        let ph = fbm(x, CHUNK_S1, z, CHUNK_S1, self.seed+5, 0.005);
        let params = BiomeParameters {
            continentalness: &continentalness, 
            mountainness: &mountainness, 
            temperature: &temperature, 
            humidity: &humidity
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
                    let block = dominant_block.unwrap();
                    let layer_width = height-last_height;
                    if block == Block::GrassBlock {
                        world.set_yrange(col, (dx, dz), height, 1, block);
                        if layer_width > 1 {
                            world.set_yrange(col, (dx, dz), height-1, (layer_width-1) as usize, Block::Dirt);
                        }
                    } else {
                        world.set_yrange(col, (dx, dz), height, layer_width as usize, block);
                    }
                    last_height = height;
                }
            }
        }
        // TODO: add back vegetation
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
            let rng = <BlockPos2d>::from((col, spot)).prng(self.seed as i32);
            let dx = spot.0 + (rng & 0b111);
            let dz = spot.1 + ((rng >> 3) & 0b111);
            let i = dx*CHUNK_S1 + dz;
            let tree = trees[i];
            if tree < 0.5 {
                continue;
            }
            let h = (rng >> 5) & 0b11;
            let (block, y) = world.top_block((col, (dx, dz)).into());
            if !block.is_fertile_soil() {
                continue;
            }
            let (tree, dist) = self.plant_ranges.closest([
                temperature[i],
                humidity[i],
                ph[i],
                y as f32 / MAX_GEN_HEIGHT as f32,
            ]);
            if dist >= 0. {
                let pos = BlockPos {
                    x: col.x * CHUNK_S1I + dx as i32,
                    y,
                    z: col.z * CHUNK_S1I + dz as i32,
                    realm: col.realm,
                };
                tree.grow(world, pos, self.seed as i32, dist + h as f32 / 10.);
            }
        }
    }
}