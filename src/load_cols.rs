use crate::blocs::{Blocs, ChunkPos2D};
use crate::col_commands::ColCommands;
use crate::terrain_gen::Generators;
use bevy::prelude::*;
use itertools::Itertools;

#[derive(Event)]
pub struct ColLoadEvent(pub ChunkPos2D);

#[derive(Event)]
pub struct ColUnloadEvent(pub ChunkPos2D);


pub fn pull_orders(
    mut col_commands: ResMut<ColCommands>,
    mut blocs: ResMut<Blocs>,
    gens: Res<Generators>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
    mut ev_load: EventWriter<ColLoadEvent>,
) {
    // LOAD ORDERS
    let mut load_orders = col_commands.loads.drain().collect_vec();
    // just take 1 generation order at a time to spread the work over multiple frames
    if let Some(pos) = load_orders.pop() {
        blocs.0.insert(pos, gens.gen(pos));
        ev_load.send(ColLoadEvent(pos));
    }
    col_commands.loads = load_orders.into_iter().collect();
    // UNLOAD ORDERS
    for pos in col_commands.unloads.drain() {
        blocs.0.remove(&pos);
        ev_unload.send(ColUnloadEvent(pos));
    }    
}
