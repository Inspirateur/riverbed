use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::blocs::{Blocs, ColPos, Realm, ReinsertTrait};
use itertools::Itertools;
use bevy::prelude::*;
use parking_lot::RwLock;
use super::{LoadArea, RenderDistance};


#[derive(Resource)]
pub struct LoadOrders {
    // { column: { player } }
    player_cols: HashMap<ColPos, HashSet<u32>>,
    // [(column, min dist to player)]
    pub to_generate: Arc<RwLock<Vec<(ColPos, u32)>>>,
    pub to_unload: Vec<ColPos>,
}

impl LoadOrders {
    pub fn new() -> Self {
        LoadOrders {
            player_cols: HashMap::new(),
            to_generate: Arc::new(RwLock::new(Vec::new())),
            to_unload: Vec::new(),
        }
    }

    fn unload_col(&mut self, col_pos: ColPos) {
        self.player_cols.remove(&col_pos);
        // NOTE: very important to store this in an intermediary variable 
        // or else the read lock leaves long enough that we reach the write lock in the if
        let generate_order_opt = self.to_generate.read_arc()
            .iter()
            .find_position(|(pos_, _)| *pos_ == col_pos)
            .and_then(|(i, _)| Some(i));
        if let Some(i) = generate_order_opt {
            // the column was still waiting for load
            self.to_generate.write_arc().remove(i);
        } else {
            self.to_unload.push(col_pos);
        }
    }

    fn add_gen_order(&mut self, col_pos: ColPos, dist: u32) {
        // col_pos should *not* be present in to_generate
        // need to take a write lock before doing read and write or else to_generate could change between the read and the write
        let mut wlock = self.to_generate.write_arc();
        let i = match wlock.binary_search_by(|(_, other_dist)| dist.cmp(other_dist)) {
            Ok(i) => i,
            Err(i) => i
        };
        wlock.insert(i, (col_pos, dist));
    }

    fn update_gen_order(&mut self, col_pos: &ColPos, dist: u32) {
        // col_pos may be present in to_generate
        let Some(old_i) = self.to_generate.read_arc().iter().position(|(other_col, _)| other_col == col_pos) else {
            return;
        };
        let new_i = match self.to_generate.read_arc().binary_search_by(|(_, other_dist)| dist.cmp(other_dist)) {
            Ok(i) => i,
            Err(i) => i
        };
        self.to_generate.write_arc().reinsert(old_i, new_i);
    }

    pub fn on_load_area_change(&mut self, player_id: u32, old_load_area: &LoadArea, new_load_area: &LoadArea) {
        for col_pos in old_load_area.col_dists.keys() {
            if new_load_area.col_dists.contains_key(col_pos) {
                continue;
            }
            if let Some(players) = self.player_cols.get_mut(&col_pos) {
                players.remove(&player_id);
                if players.len() == 0 {
                    self.unload_col(*col_pos);
                }
            }
        }
        for (col_pos, dist) in new_load_area.col_dists.iter() {
            if old_load_area.col_dists.contains_key(col_pos) {
                continue;
            }
            let players = self.player_cols.entry(*col_pos).or_insert_with(|| HashSet::new());
            let is_new = players.len() == 0;
            players.insert(player_id);
            if is_new {
                self.add_gen_order(*col_pos, *dist);
            } else {
                self.update_gen_order(col_pos, *dist)
            }
        }
    }
}

pub fn assign_load_area(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Realm, &RenderDistance)>, 
    mut col_orders: ResMut<LoadOrders>
) {
    let (player, transform, realm, render_dist) = query.single_mut();
    let col = ColPos::from((transform.translation, *realm));
    let old_load_area = LoadArea::empty();
    let new_load_area = LoadArea::new( col, *render_dist);
    col_orders.on_load_area_change(player.index(), &old_load_area, &new_load_area);
    commands.insert_resource(new_load_area.clone());  
}

pub fn update_load_area(
    mut query: Query<(Entity, &Transform, &Realm, &RenderDistance)>, 
    mut col_orders: ResMut<LoadOrders>, 
    mut load_area: ResMut<LoadArea>
) {
    for (player, transform, realm, render_dist) in query.iter_mut() {
        let col = ColPos::from((transform.translation, *realm));
        // we're checking before modifying to avoid triggering unnecessary Change detection
        if col != load_area.center {
            let new_load_area = LoadArea::new( col, *render_dist);
            col_orders.on_load_area_change(player.index(), &load_area, &new_load_area);
            *load_area = new_load_area;
        }
    }
}

pub fn on_render_distance_change(
    mut query: Query<(Entity, &RenderDistance), Changed<RenderDistance>>, 
    mut col_orders: ResMut<LoadOrders>, 
    mut load_area: ResMut<LoadArea>) {
    for (player, render_dist) in query.iter_mut() {
        let new_load_area = LoadArea::new( load_area.center, *render_dist);
        col_orders.on_load_area_change(player.index(), &load_area, &new_load_area);
        *load_area = new_load_area;
    }
}

#[derive(Event)]
pub struct ColUnloadEvent(pub ColPos);

pub fn process_unload_orders(
    mut col_orders: ResMut<LoadOrders>,
    blocs: ResMut<Blocs>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
) {
    // PROCESS UNLOAD ORDERS
    for col in col_orders.to_unload.drain(..) {
        blocs.unload_col(col);
        ev_unload.send(ColUnloadEvent(col));
    }
}