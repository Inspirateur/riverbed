use crate::bloc_pos::ChunkPos2D;
use crate::blocs::Blocs;
use crate::col_commands::ColCommands;
use crate::terrain_gen::Generators;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::sync::Arc;

#[derive(Component)]
pub struct LoadChunks(Task<Blocs>);
pub struct ColUnloadEvent(pub ChunkPos2D);

pub fn pull_orders(
    mut commands: Commands,
    mut col_commands: ResMut<ColCommands>,
    mut blocs: ResMut<Blocs>,
    gens: Res<Arc<Generators>>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
) {
    // LOAD ORDERS
    let load_orders: Vec<_> = col_commands.loads.drain().collect();
    if load_orders.len() > 0 {
        let thread_pool = AsyncComputeTaskPool::get();
        let gens = gens.clone();
        let task = thread_pool.spawn(async move {
            let mut res = Blocs::new();
            for pos in load_orders {
                res.insert_col(pos, gens.gen(pos));
            }
            res
        });
        commands.spawn().insert(LoadChunks(task));
    }
    // UNLOAD ORDERS
    let unload_orders: Vec<_> = col_commands.unloads.drain().collect();
    if unload_orders.len() > 0 {
        ev_unload.send_batch(unload_orders.into_iter().filter_map(|pos| {
            if !blocs.remove_col(pos) {
                col_commands.unloads.insert(pos);
                None
            } else {
                Some(ColUnloadEvent(pos))
            }
        }));
    }
}

pub struct ColLoadEvent(pub ChunkPos2D);

pub fn poll_gen(
    mut commands: Commands,
    mut load_tasks: Query<(Entity, &mut LoadChunks)>,
    mut world: ResMut<Blocs>,
    mut ev_load: EventWriter<ColLoadEvent>,
) {
    for (entity, mut task) in &mut load_tasks {
        if let Some(chunks) = future::block_on(future::poll_once(&mut task.0)) {
            ev_load.send_batch(chunks.cols().map(|(k, _)| ColLoadEvent(k)));
            world.extend(chunks);
            commands.entity(entity).remove::<LoadChunks>();
        }
    }
}
