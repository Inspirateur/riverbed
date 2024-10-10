use super::BlockPos;
use super::{
    pos2d::Pos2d, utils::ReinsertTrait, ColPos, PlayerArea, Realm, RenderDistance, VoxelWorld,
    CHUNK_S1,
};
use bevy::prelude::*;
use itertools::Itertools;
use parking_lot::lock_api::ArcRwLockWriteGuard;
use parking_lot::{RawRwLock, RwLock};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn add_gen_order(
    to_generate: &mut ArcRwLockWriteGuard<RawRwLock, Vec<(Pos2d<CHUNK_S1>, u32)>>,
    col_pos: ColPos,
    dist: u32,
) {
    // col_pos should *not* be present in to_generate
    // need to take a write lock before doing read and write or else to_generate could change between the read and the write
    let i = match to_generate.binary_search_by(|(_, other_dist)| dist.cmp(other_dist)) {
        Ok(i) => i,
        Err(i) => i,
    };
    to_generate.insert(i, (col_pos, dist));
}

fn update_gen_order(
    to_generate: &mut ArcRwLockWriteGuard<RawRwLock, Vec<(Pos2d<CHUNK_S1>, u32)>>,
    col_pos: &ColPos,
    dist: u32,
) {
    // col_pos may be present in to_generate
    let Some(old_i) = to_generate
        .iter()
        .position(|(other_col, _)| other_col == col_pos)
    else {
        return;
    };
    let new_i = match to_generate.binary_search_by(|(_, other_dist)| dist.cmp(other_dist)) {
        Ok(i) => i,
        Err(i) => i,
    };
    to_generate.reinsert(old_i, new_i);
}

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
        // or else the read lock lives long enough that we reach the write lock in the if
        let generate_order_opt = self
            .to_generate
            .read_arc()
            .iter()
            .find_position(|(pos_, _)| *pos_ == col_pos)
            .map(|(i, _)| i);
        if let Some(i) = generate_order_opt {
            // the column was still waiting for load
            self.to_generate.write_arc().remove(i);
        } else {
            self.to_unload.push(col_pos);
        }
    }

    pub fn on_load_area_change(
        &mut self,
        player_id: u32,
        old_load_area: &PlayerArea,
        new_load_area: &PlayerArea,
    ) {
        for col_pos in old_load_area.col_dists.keys() {
            if new_load_area.col_dists.contains_key(col_pos) {
                continue;
            }
            if let Some(players) = self.player_cols.get_mut(col_pos) {
                players.remove(&player_id);
                if players.is_empty() {
                    self.unload_col(*col_pos);
                }
            }
        }
        let mut wlock: ArcRwLockWriteGuard<RawRwLock, Vec<(Pos2d<CHUNK_S1>, u32)>> =
            self.to_generate.write_arc();
        for (col_pos, dist) in new_load_area.col_dists.iter() {
            if old_load_area.col_dists.contains_key(col_pos) {
                continue;
            }
            let players = self.player_cols.entry(*col_pos).or_default();
            let is_new = players.is_empty();
            players.insert(player_id);
            if is_new {
                add_gen_order(&mut wlock, *col_pos, *dist);
            } else {
                update_gen_order(&mut wlock, col_pos, *dist)
            }
        }
    }
}

pub fn assign_load_area(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Realm, &RenderDistance)>,
    mut col_orders: ResMut<LoadOrders>,
) {
    let (player, transform, realm, render_dist) = query.single_mut();
    let col = ColPos::from((transform.translation, *realm));
    let old_load_area = PlayerArea::empty();
    let new_load_area = PlayerArea::new(col, *render_dist);
    col_orders.on_load_area_change(player.index(), &old_load_area, &new_load_area);
    commands.insert_resource(new_load_area.clone());
}

pub fn update_load_area(
    mut query: Query<(Entity, &Transform, &Realm, &RenderDistance)>,
    mut col_orders: ResMut<LoadOrders>,
    mut load_area: ResMut<PlayerArea>,
) {
    for (player, transform, realm, render_dist) in query.iter_mut() {
        let col = ColPos::from((transform.translation, *realm));
        // we're checking before modifying to avoid triggering unnecessary Change detection
        if col != load_area.center {
            let new_load_area = PlayerArea::new(col, *render_dist);
            col_orders.on_load_area_change(player.index(), &load_area, &new_load_area);
            *load_area = new_load_area;
        }
    }
}

pub fn on_render_distance_change(
    mut query: Query<(Entity, &RenderDistance), Changed<RenderDistance>>,
    mut col_orders: ResMut<LoadOrders>,
    mut load_area: ResMut<PlayerArea>,
) {
    for (player, render_dist) in query.iter_mut() {
        let new_load_area = PlayerArea::new(load_area.center, *render_dist);
        col_orders.on_load_area_change(player.index(), &load_area, &new_load_area);
        *load_area = new_load_area;
    }
}

#[derive(Default, Resource)]
pub struct BlockEntities(HashMap<ColPos, HashMap<(usize, i32, usize), Entity>>);

impl BlockEntities {
    pub fn unload_col(&mut self, col_pos: &ColPos) -> Vec<Entity> {
        let Some(entities) = self.0.remove(col_pos) else {
            return Vec::new()
        };
        entities.into_values().into_iter().collect()
    }

    pub fn get(&self, block_pos: &BlockPos) -> Option<Entity> {
        let (col_pos, pos) = (*block_pos).into();
        let col_ents = self.0.get(&col_pos)?;
        col_ents.get(&pos).copied()
    }

    pub fn remove(&mut self, block_pos: &BlockPos) {
        let (col_pos, pos) = (*block_pos).into();
        let Some(entities) = self.0.get_mut(&col_pos) else {
            return;
        };
        entities.remove(&pos);
    }

    pub fn add(&mut self, block_pos: &BlockPos, entity: Entity) {
        let (col_pos, pos) = (*block_pos).into();
        self.0.entry(col_pos).or_default().insert(pos, entity);
    }
}

#[derive(Event)]
pub struct ColUnloadEvent(pub ColPos);

pub fn process_unload_orders(
    mut commands: Commands,
    mut col_orders: ResMut<LoadOrders>,
    blocks: ResMut<VoxelWorld>,
    mut ev_unload: EventWriter<ColUnloadEvent>,
    mut col_entities: ResMut<BlockEntities>,
) {
    // PROCESS UNLOAD ORDERS
    for col in col_orders.to_unload.drain(..) {
        blocks.unload_col(col);
        for entity_id in col_entities.unload_col(&col) {
            if let Some(mut entity) = commands.get_entity(entity_id) {
                entity.despawn();
            }
        }
        ev_unload.send(ColUnloadEvent(col));
    }
}
