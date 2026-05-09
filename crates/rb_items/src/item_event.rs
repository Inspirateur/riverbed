use bevy::prelude::EntityEvent;

/// Triggered on an entity when it picks up an item.
#[derive(EntityEvent)]
pub struct ItemGet {
    pub entity: bevy::prelude::Entity,
}
