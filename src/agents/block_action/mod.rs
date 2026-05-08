mod block_hit_place;
mod furnace_action;
mod split;
pub use furnace_action::*;
pub use block_hit_place::*;
use bevy::prelude::*;
use block_hit_place::BlockHitPlacePlugin;
use furnace_action::FurnaceActionPlugin;
use split::{SplitChannel, apply_split_outputs, schedule_split_jobs};

pub struct BlockActionPlugin;

impl Plugin for BlockActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BlockHitPlacePlugin, FurnaceActionPlugin))
            .init_resource::<SplitChannel>()
            .add_systems(Update, (schedule_split_jobs, apply_split_outputs).chain());
    }
}