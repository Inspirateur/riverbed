use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{Receiver, Sender, unbounded};
use rb_generation::{Biome, TerrainGenerator};
use rb_pos::{ChunkPos2d, Pos2d};
use rb_world::{ColUnloadEvent, VoxelWorld, WorldRng, chunk_area};
const LOAD_RADIUS: u32 = 6;

pub struct BiomeTerrainLoaderPlugin;

impl Plugin for BiomeTerrainLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ColUnloadEvent>()
            .insert_resource(TargetBiome(Biome::Canyon))
            .add_systems(Startup, setup_recievers)
            .add_systems(Update, setup_load_thread)
            .add_systems(Update, on_unload_col);
    }
}

#[derive(Debug, Clone, Copy, Resource)]
struct TargetBiome(Biome);

fn setup_recievers(mut commands: Commands) {
    let (unload_sender, unload_recv) = unbounded::<ChunkPos2d>();
    commands.insert_resource(ColUnloadSender(unload_sender));
    commands.insert_resource(ColUnloadsReciever(unload_recv));
}

fn setup_load_thread(
    world: Res<VoxelWorld>,
    world_rng: Res<WorldRng>,
    target_biome: Res<TargetBiome>,
    unload_sender: Res<ColUnloadSender>,
) {
    if !target_biome.is_changed() {
        return;
    }
    println!("Loading biome {:?}", target_biome.0);
    let thread_pool = AsyncComputeTaskPool::get();
    let load_world = world.clone();
    let seed_value = world_rng.seed;
    let biome = target_biome.0;
    let unload_sender = unload_sender.0.clone();
    thread_pool
        .spawn(async move {
            // first unload anything that might be loaded
            while let Some(col) = load_world.loaded_columns.pop_back() {
                load_world.unload_col(*col);
                if unload_sender.send(*col).is_err() {
                    // This means the game is shutting down, so we break the loop
                    warn!("ColUnloadsReciever channel is closed, stopping terrain thread");
                    break;
                }
            }
            // Deal with unloaded world columns that have data
            // (happens when a structure generate blocks in a chunk that was not supposed to be loaded)
            while let Some(col) = load_world.unloaded_columns.pop_back() {
                load_world.unload_col(*col);
                if unload_sender.send(*col).is_err() {
                    // This means the game is shutting down, so we break the loop
                    warn!("ColUnloadsReciever channel is closed, stopping terrain thread");
                    break;
                }
            }
            // load new terrain
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
            let mut col_to_load =
                chunk_area(&Pos2d::default(), LOAD_RADIUS as i32).collect::<Vec<_>>();
            col_to_load.sort_by_key(|pos| pos.dist(Pos2d::default()));
            for col in col_to_load {
                let mut col_params = terrain_gen.biome_params_at(col);
                for (i, param) in terrain_gen.biomes_points.parameters.iter().enumerate() {
                    col_params
                        .0
                        .get_mut(param)
                        .unwrap()
                        .iter_mut()
                        .for_each(|v| *v += bias_params[i]);
                }
                load_world.loaded_columns.insert(col);
                terrain_gen.generate_with_params(&load_world, col.into(), col_params);
                load_world.mark_change_col(col);
            }
        })
        .detach();
}

#[derive(Resource)]
pub struct ColUnloadSender(pub Sender<ChunkPos2d>);

#[derive(Resource)]
pub struct ColUnloadsReciever(pub Receiver<ChunkPos2d>);

pub fn on_unload_col(
    unload_cols: Res<ColUnloadsReciever>,
    mut unload_event: MessageWriter<ColUnloadEvent>,
) {
    while let Ok(col) = unload_cols.0.try_recv() {
        unload_event.write(ColUnloadEvent(col));
    }
}
