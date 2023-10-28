use std::collections::{HashMap, HashSet};
use crate::blocs::{Blocs, ChunkPos2D, Realm};
use crate::gen::{Generators, load_area::{update_load_area, load_order}};
use itertools::Itertools;
use bevy::prelude::*;


#[derive(Resource)]
pub struct LoadedCols {
    // a hashmap of chunk columns and their players
    cols: HashMap<ChunkPos2D, HashSet<u32>>,
    pub loads: Vec<ChunkPos2D>,
    pub unloads: Vec<ChunkPos2D>,
}

impl LoadedCols {
    pub fn new() -> Self {
        LoadedCols {
            cols: HashMap::new(),
            loads: Vec::new(),
            unloads: Vec::new(),
        }
    }

    pub fn has_player(&self, pos: ChunkPos2D) -> bool {
        self.cols.contains_key(&pos)
    }

    pub fn register(&mut self, to_load: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_load.into_iter() {
            let pos = ChunkPos2D { realm, x, z };
            let players = self.cols.entry(pos).or_insert_with(|| HashSet::new());
            if players.len() == 0 {
                self.loads.push(pos);
            }
            players.insert(player);
        }
    }

    pub fn unregister(&mut self, to_unload: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_unload.into_iter() {
            let pos = ChunkPos2D { realm, x, z };
            let players = self.cols.entry(pos).or_insert_with(|| HashSet::new());
            players.remove(&player);
            if players.len() == 0 {
                self.cols.remove(&pos);
                if let Some((i, _)) = self.loads.iter().find_position(|pos_| **pos_ == pos) {
                    // the column was still waiting for load
                    self.loads.remove(i);
                } else {
                    self.unloads.push(pos);
                }
            }
        }
    }
}


#[derive(Event)]
pub struct ColUnloadEvent(pub ChunkPos2D);


pub fn pull_orders(
    mut col_commands: ResMut<LoadedCols>,
    mut blocs: ResMut<Blocs>,
    gens: Res<Generators>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
) {
    // PROCESS UNLOAD ORDERS
    for col in col_commands.unloads.drain(..) {
        blocs.unload_col(col);
        ev_unload.send(ColUnloadEvent(col));
    }
    // take 1 generation order at a time to spread the work over multiple frames
    if let Some(col) = col_commands.loads.pop() {
        gens.gen(&mut blocs, col);
        blocs.register(col)
    }
}


pub struct LoadTerrainPlugin;

impl Plugin for LoadTerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(LoadedCols::new())
            .insert_resource(Generators::new(0))
            .add_event::<ColUnloadEvent>()
            .add_systems(Update, update_load_area)
            .add_systems(Update, load_order)
            .add_systems(Update, pull_orders)
        ;
    }
}