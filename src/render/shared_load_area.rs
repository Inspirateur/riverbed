use std::sync::Arc;
use parking_lot::RwLock;
use bevy::ecs::{change_detection::DetectChanges, resource::Resource, system::{Commands, Res}};
use crate::world::PlayerArea;


#[derive(Resource)]
pub struct SharedLoadArea(pub Arc<RwLock<PlayerArea>>);

pub fn setup_shared_load_area(mut commands: Commands, load_area: Res<PlayerArea>) {
    commands.insert_resource(SharedLoadArea(Arc::new(RwLock::new(load_area.clone()))))
}

pub fn update_shared_load_area(load_area: Res<PlayerArea>, shared_load_area: Res<SharedLoadArea>) {
    if !load_area.is_changed() { return; }
    *shared_load_area.0.write() = load_area.clone();
}