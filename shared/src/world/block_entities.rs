use bevy::prelude::*;
use hashbrown::HashMap;

use crate::world::pos::{BlockPos, ColPos};

#[derive(Default, Clone, Resource)]
pub struct BlockEntities(pub HashMap<ColPos, HashMap<(usize, i32, usize), Entity>>);

pub trait BlockEntitiesTrait {
    fn unload_col(&mut self, col_pos: &ColPos) -> Vec<Entity>;
    fn get(&self, block_pos: &BlockPos) -> Option<Entity>;
    fn remove(&mut self, block_pos: &BlockPos);
    fn add(&mut self, block_pos: &BlockPos, entity: Entity);
}
