use std::collections::{HashMap, HashSet};

use crate::bloc::Bloc;
use crate::chunk;
use crate::chunk::Chunk;
use crate::realm::Realm;
// i32 is a more convenient format here
const CHUNK_S1: i32 = chunk::CHUNK_S1 as i32;
pub struct WorldData {
    // a hashmap of chunk columns and their players
    cols: HashMap<(Realm, i32, i32), HashSet<u32>>,
    // the Chunk data
    chunks: HashMap<(Realm, i32, i32, i32), Chunk>,
    pub load_orders: HashSet<(Realm, i32, i32)>,
    pub unload_orders: HashSet<(Realm, i32, i32)>,
}

impl WorldData {
    pub fn new() -> Self {
        WorldData {
            cols: HashMap::new(),
            chunks: HashMap::new(),
            load_orders: HashSet::new(),
            unload_orders: HashSet::new(),
        }
    }
    pub fn register_load(&mut self, to_load: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_load.into_iter() {
            let key = (realm, x, z);
            if let Some(players) = self.cols.get_mut(&key) {
                players.insert(player);
            } else {
                let mut players = HashSet::new();
                players.insert(player);
                self.cols.insert(key, players);
                println!("loading {:?}", key);
                self.load_orders.insert(key);
            }
        }
    }

    pub fn register_unload(&mut self, to_unload: Vec<(i32, i32)>, realm: Realm, player: u32) {
        for (x, z) in to_unload.into_iter() {
            let key = (realm, x, z);
            if let Some(players) = self.cols.get_mut(&key) {
                players.remove(&player);
                if players.len() == 0 {
                    self.cols.remove(&key);
                    println!("unloading {:?}", key);
                    self.unload_orders.insert(key);
                }
            }
        }
    }

    pub fn get(&self, realm: Realm, x: i32, y: i32, z: i32) -> Bloc {
        let (qx, qy, qz) = (x / CHUNK_S1, y / CHUNK_S1, z / CHUNK_S1);
        let (rx, ry, rz) = (x % CHUNK_S1, y % CHUNK_S1, z % CHUNK_S1);
        if let Some(chunk) = self.chunks.get(&(realm, qx, qy, qz)) {
            *chunk.get(rx as usize, ry as usize, rz as usize)
        } else {
            Bloc::Air
        }
    }

    pub fn set(&mut self, realm: Realm, x: i32, y: i32, z: i32, bloc: Bloc) {
        let (qx, qy, qz) = (x / CHUNK_S1, y / CHUNK_S1, z / CHUNK_S1);
        let (rx, ry, rz) = (x % CHUNK_S1, y % CHUNK_S1, z % CHUNK_S1);
        let chunk_pos = (realm, qx, qy, qz);
        // The chunk has to be loaded at this point !
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set(rx as usize, ry as usize, rz as usize, bloc);
        }
    }
}
