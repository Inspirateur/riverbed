use std::collections::{HashMap, HashSet, VecDeque};
use crate::blocs::{Blocs, ColPos, Realm};
use crate::gen::Generators;
use itertools::Itertools;
use bevy::prelude::*;


#[derive(Resource)]
pub struct LoadedCols {
    // a hashmap of chunk columns and their players
    cols: HashMap<ColPos, HashSet<u32>>,
    pub loads: VecDeque<ColPos>,
    pub unloads: Vec<ColPos>,
}

impl LoadedCols {
    pub fn new() -> Self {
        LoadedCols {
            cols: HashMap::new(),
            loads: VecDeque::new(),
            unloads: Vec::new(),
        }
    }

    pub fn in_player_range(&self, pos: ColPos) -> bool {
        self.cols.contains_key(&pos)
    }

    pub fn register(&mut self, to_load: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_load.into_iter() {
            let pos = ColPos { realm, x, z };
            let players = self.cols.entry(pos).or_insert_with(|| HashSet::new());
            if players.len() == 0 {
                self.loads.push_back(pos);
            }
            players.insert(player);
        }
    }

    pub fn unregister(&mut self, to_unload: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_unload.into_iter() {
            let pos = ColPos { realm, x, z };
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
pub struct ColUnloadEvent(pub ColPos);

pub fn process_unload_orders(
    mut col_commands: ResMut<LoadedCols>,
    mut blocs: ResMut<Blocs>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
) {
    // PROCESS UNLOAD ORDERS
    for col in col_commands.unloads.drain(..) {
        blocs.unload_col(col);
        ev_unload.send(ColUnloadEvent(col));
    }
}

pub fn process_load_order(
    mut col_commands: ResMut<LoadedCols>,
    mut blocs: ResMut<Blocs>,
    gens: Res<Generators>,
) {
    // take 1 generation order at a time to spread the work over multiple frames
    if let Some(col) = col_commands.loads.pop_front() {
        gens.gen(&mut blocs, col);
        blocs.register(col)
    }
}
