mod block_hit_place;
mod furnace_action;
pub use furnace_action::*;
pub use block_hit_place::*;
use bevy::prelude::*;
use block_hit_place::BlockHitPlacePlugin;
use furnace_action::FurnaceActionPlugin;

pub struct BlockActionPlugin;

impl Plugin for BlockActionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                BlockHitPlacePlugin,
                FurnaceActionPlugin
            ))
        ;
    }
}