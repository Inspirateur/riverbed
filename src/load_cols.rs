use std::collections::VecDeque;
use ourcraft::{Blocs, ChunkPos2D};
use crate::col_commands::ColCommands;
use crate::terrain_gen::Generators;
use bevy::prelude::*;

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
    // PROCESS UNLOAD ORDERS
    for col in col_commands.unloads.drain(..) {
        blocs.unload_col(col);
        ev_unload.send(ColUnloadEvent(col));
    }
    // take 1 generation order at a time to spread the work over multiple frames
    if let Some(pos) = col_commands.loads.pop() {
        gens.gen(&mut blocs, pos);
        ev_load.0.push_front(pos);
    }
}
