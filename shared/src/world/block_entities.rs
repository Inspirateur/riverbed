use bevy::prelude::*;
use hashbrown::HashMap;

use crate::world::pos::{BlockPos, ColPos};

#[derive(Default, Clone, Resource)]
pub struct BlockEntities(HashMap<ColPos, HashMap<(usize, i32, usize), Entity>>);

impl BlockEntities {
    pub fn unload_col(&mut self, col_pos: &ColPos) -> Vec<Entity> {
        let Some(entities) = self.0.remove(col_pos) else {
            return Vec::new()
        };
        entities.into_values().into_iter().collect()
    }

    pub fn get(&self, block_pos: &BlockPos) -> Option<Entity> {
        let (col_pos, pos): (ColPos, (usize, i32, usize)) = (*block_pos).into();
        let col_ents = self.0.get(&col_pos)?;
        col_ents.get(&pos).copied()
    }

    pub fn remove(&mut self, block_pos: &BlockPos) {
        let (col_pos, pos): (ColPos, (usize, i32, usize)) = (*block_pos).into();
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