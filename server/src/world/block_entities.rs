use bevy::prelude::*;
use shared::world::block_entities::BlockEntities;

use crate::world::ColUnloadEvent;


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