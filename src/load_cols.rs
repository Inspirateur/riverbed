use crate::chunk_map::ChunkMap;
use crate::realm::Realm;
use crate::terrain_gen::TerrainGen;
use crate::world_data::WorldData;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use dashmap::DashMap;
use futures_lite::future;
use std::sync::Arc;

#[derive(Component)]
pub struct LoadChunks(Task<ChunkMap>);
pub struct ColUnloadEvent(pub (Realm, i32, i32));

pub fn pull_orders(
    mut commands: Commands,
    mut world: ResMut<WorldData>,
    gens: Res<Arc<DashMap<Realm, Box<dyn TerrainGen>>>>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
) {
    // UNLOAD ORDERS
    let unload_orders: Vec<_> = world.unload_orders.drain().collect();
    if unload_orders.len() > 0 {
        for (realm, x, z) in unload_orders.iter() {
            world.chunks.remove_col(*realm, *x, *z);
        }
        ev_unload.send_batch(unload_orders.into_iter().map(|k| ColUnloadEvent(k)));
    }
    // LOAD ORDERS
    let load_orders: Vec<_> = world.load_orders.drain().collect();
    if load_orders.len() > 0 {
        let thread_pool = AsyncComputeTaskPool::get();
        let gens = gens.clone();
        let task = thread_pool.spawn(async move {
            let mut res = ChunkMap::new();
            for (realm, x, z) in load_orders {
                for (y, chunk) in gens.get(&realm).unwrap().gen((x, z)) {
                    res.insert(realm, x, y, z, chunk);
                }
            }
            res
        });
        commands.spawn().insert(LoadChunks(task));
    }
}

pub struct ColLoadEvent(pub (Realm, i32, i32));

pub fn poll_gen(
    mut commands: Commands,
    mut load_tasks: Query<(Entity, &mut LoadChunks)>,
    mut world: ResMut<WorldData>,
    mut ev_load: EventWriter<ColLoadEvent>,
) {
    for (entity, mut task) in &mut load_tasks {
        if let Some(chunks) = future::block_on(future::poll_once(&mut task.0)) {
            ev_load.send_batch(chunks.cols().map(|(k, _)| ColLoadEvent(k)));
            world.chunks.extend(chunks);
            commands.entity(entity).remove::<LoadChunks>();
        }
    }
}
