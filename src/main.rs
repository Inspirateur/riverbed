mod bloc;
mod chunk;
mod chunk_map;
mod draw2d;
mod earth_gen;
mod get_set;
mod load_area;
mod load_cols;
mod noise_utils;
mod packed_ints;
mod player;
mod pos;
mod realm;
mod terrain_gen;
mod weighted_dist;
mod world_data;
use bevy::prelude::*;
use draw2d::Draw2d;
use leafwing_input_manager::plugin::InputManagerPlugin;
use load_cols::{ColLoadEvent, ColUnloadEvent};
use std::sync::Arc;
use world_data::WorldData;

struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldData::new())
            .insert_resource(Arc::new(terrain_gen::generators(0)))
            .add_event::<ColLoadEvent>()
            .add_event::<ColUnloadEvent>()
            .add_startup_system(player::spawn_player)
            .add_system(player::move_player)
            .add_system(load_area::update_load_area)
            .add_system(load_area::load_order)
            .add_system(load_cols::pull_orders)
            .add_system(load_cols::poll_gen);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<player::Action>::default())
        .add_plugin(GameLogic)
        .add_plugin(Draw2d)
        .run();
}
