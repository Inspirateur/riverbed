mod block_hit_place;
mod furnace_action;
use bevy::prelude::*;
pub use block_hit_place::*;
pub use furnace_action::*;

pub struct BlockActionPlugin;

impl Plugin for BlockActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BlockHitPlacePlugin, FurnaceActionPlugin));
    }
}
