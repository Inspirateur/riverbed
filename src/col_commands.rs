use crate::pos::{ChunkPos2D, Realm};
use crate::blocs::Cols;
use std::collections::{HashSet};
pub const WATER_H: i32 = 100;

pub struct ColCommands {
    // a hashmap of chunk columns and their players
    cols: Cols<HashSet<u32>>,
    pub loads: HashSet<ChunkPos2D>,
    pub unloads: HashSet<ChunkPos2D>,
}

impl ColCommands {
    pub fn new() -> Self {
        ColCommands {
            cols: Cols::new(),
            loads: HashSet::new(),
            unloads: HashSet::new(),
        }
    }

    pub fn has_player(&self, pos: ChunkPos2D) -> bool {
        self.cols.contains_key(&pos)
    }

    pub fn register(&mut self, to_load: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_load.into_iter() {
            let pos = ChunkPos2D { realm, x, z };
            let players = self.cols.entry(pos).or_insert(HashSet::new());
            if players.len() == 0 {
                self.loads.insert(pos);
            }
            players.insert(player);
        }
    }

    pub fn unregister(&mut self, to_unload: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_unload.into_iter() {
            let pos = ChunkPos2D { realm, x, z };
            let players = self.cols.entry(pos).or_insert(HashSet::new());
            players.remove(&player);
            if players.len() == 0 {
                self.cols.remove(&pos);
                println!("unloading {:?}", pos);
                self.unloads.insert(pos);
            }
        }
    }
}
