use crate::{bloc_pos::ChunkPos2D, realm::Realm};
use std::collections::{HashMap, HashSet};
pub const WATER_H: i32 = 100;

pub struct ColCommands {
    // a hashmap of chunk columns and their players
    cols: HashMap<ChunkPos2D, HashSet<u32>>,
    pub loads: HashSet<ChunkPos2D>,
    pub unloads: HashSet<ChunkPos2D>,
}

impl ColCommands {
    pub fn new() -> Self {
        ColCommands {
            cols: HashMap::new(),
            loads: HashSet::new(),
            unloads: HashSet::new(),
        }
    }

    pub fn load(&mut self, to_load: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_load.into_iter() {
            let key = ChunkPos2D { realm, x, z };
            if let Some(players) = self.cols.get_mut(&key) {
                players.insert(player);
            } else {
                let mut players = HashSet::new();
                players.insert(player);
                self.cols.insert(key, players);
                self.loads.insert(key);
            }
        }
    }

    pub fn unload(&mut self, to_unload: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_unload.into_iter() {
            let key = ChunkPos2D { realm, x, z };
            if let Some(players) = self.cols.get_mut(&key) {
                players.remove(&player);
                if players.len() == 0 {
                    self.cols.remove(&key);
                    println!("unloading {:?}", key);
                    self.unloads.insert(key);
                }
            }
        }
    }
}
