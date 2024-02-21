use std::sync::Arc;
use parking_lot::RwLock;
use bevy::ecs::{change_detection::DetectChanges, system::{Commands, Res, Resource}};
use crate::gen::LoadArea;


#[derive(Resource)]
pub struct SharedLoadArea(pub Arc<RwLock<LoadArea>>);

pub fn setup_shared_load_area(mut commands: Commands, load_area: Res<LoadArea>) {
    commands.insert_resource(SharedLoadArea(Arc::new(RwLock::new(load_area.clone()))))
}

pub fn update_shared_load_area(load_area: Res<LoadArea>, shared_load_area: Res<SharedLoadArea>) {
    if load_area.is_changed() {
        *shared_load_area.0.write() = load_area.clone();
    }
}