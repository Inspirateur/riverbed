mod block_hit_place;
mod furnace_action;
use bevy::prelude::*;
use block_hit_place::BlockHitPlacePlugin;
pub use block_hit_place::*;
use furnace_action::FurnaceActionPlugin;
pub use furnace_action::*;

pub struct BlockActionPlugin;

impl Plugin for BlockActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BlockHitPlacePlugin, FurnaceActionPlugin));
    }
}
