use std::collections::{HashMap, VecDeque};
use ourcraft::{Blocs, ChunkPos2D};
use crate::col_commands::ColCommands;
use crate::terrain_gen::Generators;
use bevy::prelude::*;
use itertools::Itertools;

#[derive(Resource)]
pub struct ColEntities(HashMap::<ChunkPos2D, Vec<Entity>>);

impl ColEntities {
    pub fn new() -> Self {
        ColEntities(HashMap::new())
    }

    pub fn insert(&mut self, pos: ChunkPos2D, ent: Entity) {
        self.0.entry(pos).or_insert(Vec::new()).push(ent);
    }

    pub fn get(&self, pos: &ChunkPos2D) -> Option<&Vec<Entity>> {
        self.0.get(pos)
    }

    pub fn pop(&mut self, pos: &ChunkPos2D) -> Vec<Entity> {
        self.0.remove(pos).unwrap_or(Vec::new())
    }
}

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
    // LOAD ORDERS
    let mut load_orders = col_commands.loads.drain().collect_vec();
    // just take 1 generation order at a time to spread the work over multiple frames
    if let Some(pos) = load_orders.pop() {
        blocs.0.insert(pos, gens.gen(pos));
        ev_load.0.push_front(pos);
    }
    col_commands.loads = load_orders.into_iter().collect();
    // UNLOAD ORDERS
    for pos in col_commands.unloads.drain() {
        blocs.0.remove(&pos);
        // remove the pos from load orders queue (in case it hasn't loaded yet)
        if let Some((i, _)) = ev_load.0.iter().find_position(|_pos| _pos == &&pos) {
            ev_load.0.remove(i);
        }
        ev_unload.send(ColUnloadEvent(pos));
    }
}
