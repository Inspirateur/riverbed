use crate::{chunk::CHUNK_S1, player::Pos, realm::Realm};
use bevy::prelude::Query;
use bevy::prelude::*;

#[derive(Component)]
pub struct LoadArea {
    pub realm: Realm,
    pub col: (i32, i32),
    pub dist: u32,
}

impl LoadArea {
    pub fn contains(&self, chunk_x: i32, chunk_z: i32) -> bool {
        // checks if a chunk is in this Player loaded area (assuming they're in the same realm)
        i32::max((self.col.0 - chunk_x).abs(), (self.col.1 - chunk_z).abs()) < self.dist as i32
    }
}

pub fn update_load_area(mut query: Query<(&Realm, &Pos, &mut LoadArea), Changed<Pos>>) {
    for (realm, pos, mut load_area) in query.iter_mut() {
        let col = (
            pos.0.x as i32 / CHUNK_S1 as i32,
            pos.0.z as i32 / CHUNK_S1 as i32,
        );
        // we're checking before modifying to avoid triggering Change detection
        if *realm != load_area.realm {
            load_area.realm = *realm;
        }
        if col != load_area.col {
            load_area.col = col;
        }
    }
}

pub fn load_order(query: Query<&LoadArea, Changed<LoadArea>>) {
    for load_area in query.iter() {
        println!("Load area changed !");
    }
}
