use std::collections::VecDeque;
use ourcraft::{Blocs, ChunkPos2D, ChunkPos, Y_CHUNKS};
use crate::col_commands::ColCommands;
use crate::terrain_gen::Generators;
use bevy::prelude::*;
use itertools::Itertools;

#[derive(Resource)]
pub struct ColLoadOrders(pub VecDeque<ChunkPos2D>);

#[derive(Event)]
pub struct ColUnloadEvent(pub ChunkPos2D);


pub fn pull_orders(
    mut col_commands: ResMut<ColCommands>,
    mut blocs: ResMut<Blocs>,
    gens: Res<Generators>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
    mut ev_load: ResMut<ColLoadOrders>,
) {
    // RETRIEVE LOAD ORDERS
    let mut load_orders = col_commands.loads.drain().collect_vec();
    // PROCESS UNLOAD ORDERS
    for col in col_commands.unloads.drain() {
        blocs.unload_col(col);
        // remove the pos from load orders queue (in case it hasn't loaded yet)
        if let Some((i, _)) = load_orders.iter().find_position(|_pos| **_pos == col) {
            println!("Load Cancelled for {:?}", col);
            load_orders.remove(i);
        } else {
            ev_unload.send(ColUnloadEvent(col));
        }
    }
    // take 1 generation order at a time to spread the work over multiple frames
    if let Some(pos) = load_orders.pop() {
        gens.gen(&mut blocs, pos);
        ev_load.0.push_front(pos);
    }
    col_commands.loads = load_orders.into_iter().collect();
}
