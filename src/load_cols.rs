use crate::pos::ChunkPos2D;
use crate::blocs::Blocs;
use crate::col_commands::{ColCommands};
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
                res.insert(pos, gens.gen(pos));
            }
            res
        });
        commands.spawn().insert(LoadChunks(task));
    }
    // UNLOAD ORDERS
    for pos in col_commands.unloads.drain() {
        blocs.remove(&pos);
        ev_unload.send(ColUnloadEvent(pos));
    }    
}

pub struct ColLoadEvent(pub ChunkPos2D);

pub fn poll_gen(
    mut commands: Commands,
    col_commands: Res<ColCommands>,
    mut load_tasks: Query<(Entity, &mut LoadChunks)>,
    mut world: ResMut<Blocs>,
    mut ev_load: EventWriter<ColLoadEvent>,
) {
    for (entity, mut task) in &mut load_tasks {
        if let Some(cols) = future::block_on(future::poll_once(&mut task.0)) {
            for (pos, col) in cols.into_iter() {
                // check if the col still has player in it before adding it
                if col_commands.has_player(pos) {
                    ev_load.send(ColLoadEvent(pos));
                    world.insert(pos, col);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
