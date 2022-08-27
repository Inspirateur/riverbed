use crate::world_data::WorldData;
use bevy::prelude::*;

pub fn pull_orders(mut world: ResMut<WorldData>) {
    for col in world.unload_orders.drain() {}
    for col in world.load_orders.drain() {}
}
