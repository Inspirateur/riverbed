mod block_breaking;
use bevy::app::{Plugin, Update};
use block_breaking::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Update, add_break_animation)
            .add_systems(Update, update_break_animation)
            .add_systems(Update, remove_break_animation)
            ;
    }
}