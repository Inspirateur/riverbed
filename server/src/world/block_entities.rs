use bevy::prelude::*;
use hashbrown::HashMap;
use shared::world::block_entities::BlockEntities;

use crate::world::pos::{BlockPos, ColPos};


impl BlockEntitiesTrait for BlockEntities {
    fn unload_col(&mut self, col_pos: &ColPos) -> Vec<Entity> {
        let Some(entities) = self.0.remove(col_pos) else {
            return Vec::new()
        };
        entities.into_values().into_iter().collect()
    }

    fn get(&self, block_pos: &BlockPos) -> Option<Entity> {
        let (col_pos, pos): (ColPos, (usize, i32, usize)) = (*block_pos).into();
        let col_ents = self.0.get(&col_pos)?;
        col_ents.get(&pos).copied()
    }

    fn remove(&mut self, block_pos: &BlockPos) {
        let (col_pos, pos): (ColPos, (usize, i32, usize)) = (*block_pos).into();
        let Some(entities) = self.0.get_mut(&col_pos) else {
            return;
        };
        entities.remove(&pos);
    }

    fn add(&mut self, block_pos: &BlockPos, entity: Entity) {
        let (col_pos, pos) = (*block_pos).into();
        self.0.entry(col_pos).or_default().insert(pos, entity);
    }
}



pub fn unload_block_entities(
    mut commands: Commands,
    mut block_entities: ResMut<BlockEntities>,
    mut unload_events: EventReader<ColUnloadEvent>,
) {
    for ColUnloadEvent(col_pos) in unload_events.read() {
        let entities = block_entities.unload_col(col_pos);
        for entity in entities {
            commands.entity(entity).despawn();
        }
    }
}