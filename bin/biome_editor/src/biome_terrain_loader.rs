use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use rb_generation::{Biome, TerrainGenerator};
use rb_pos::Pos2d;
use rb_world::{VoxelWorld, WorldRng, rd_area};

pub struct BiomeTerrainLoaderPlugin;

impl Plugin for BiomeTerrainLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_load_thread);
    }
}

#[derive(Debug, Clone, Copy, Resource)]
struct TargetBiome(Biome);

fn setup_load_thread(
    world: Res<VoxelWorld>,
    world_rng: Res<WorldRng>,
    target_biome: Res<TargetBiome>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let load_world = world.clone();
    let seed_value = world_rng.seed;
    let biome = target_biome.0;
    thread_pool
        .spawn(async move {
            let terrain_gen = TerrainGenerator::new(seed_value as u32);
            let avg_params = terrain_gen
                .biome_params_at(Pos2d::default())
                .average(terrain_gen.biomes_points.parameters);
            let ideal_biome_params = terrain_gen
                .biomes_points
                .points
                .iter()
                .find(|(_, b)| *b == biome)
                .unwrap()
                .0;
            let mut bias_params = avg_params.clone();
            bias_params
                .iter_mut()
                .zip(ideal_biome_params.iter())
                .for_each(|(b, i)| *b = i - *b);
            for col in rd_area(&Pos2d::default()) {
                let mut col_params = terrain_gen.biome_params_at(col);
                for (i, param) in terrain_gen.biomes_points.parameters.iter().enumerate() {
                    col_params
                        .0
                        .get_mut(param)
                        .unwrap()
                        .iter_mut()
                        .for_each(|v| *v += bias_params[i]);
                }
                terrain_gen.generate_with_params(&load_world, col.into(), col_params);
                load_world.mark_change_col(col);
            }
        })
        .detach();
}
