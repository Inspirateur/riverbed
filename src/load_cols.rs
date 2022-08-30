use std::collections::HashMap;
use std::sync::Arc;
use crate::chunk::Chunk;
use crate::realm::Realm;
use crate::terrain_gen::TerrainGen;
use crate::world_data::WorldData;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use dashmap::DashMap;

#[derive(Component)]
pub struct LoadChunks(Task<HashMap<(Realm, i32, i32, i32), Chunk>>);

pub fn pull_orders(
    mut commands: Commands,
    mut world: ResMut<WorldData>,
    gens: Res<Arc<DashMap<Realm, Box<dyn TerrainGen>>>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (realm, x, z) in world.unload_orders.drain() {}
    let gens = gens.clone();
    let load_orders: Vec<_> = world.load_orders.drain().collect();
    let task = thread_pool.spawn(async move {
        let mut res = HashMap::new();
        for (realm, x, z) in load_orders {
            for (y, chunk) in gens.get(&realm).unwrap().gen((x, z)) {
                res.insert((realm, x, y, z), chunk);
            }
        }
        res
     });
     commands.spawn().insert(LoadChunks(task));
}

pub fn poll_gen(query: Query<&LoadChunks>) {
    
}