use crate::chunk::Chunk;
use crate::realm::Realm;
use crate::terrain_gen::TerrainGen;
use crate::world_data::WorldData;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;

#[derive(Component)]
struct LoadChunk(Task<HashMap<i32, Chunk>>);

pub fn pull_orders(
    mut commands: Commands,
    mut world: ResMut<WorldData>,
    gens: Res<HashMap<Realm, Box<dyn TerrainGen>>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (realm, x, z) in world.unload_orders.drain() {}
    for (realm, x, z) in world.load_orders.drain() {
        // let gen = gens.get(&realm).unwrap();
        // let task = thread_pool.spawn(async move { gen.gen((x, z)) });
    }
}
