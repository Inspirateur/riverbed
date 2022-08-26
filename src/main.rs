mod bloc;
mod chunk;
mod draw2d;
mod load_area;
mod noise_utils;
mod packed_ints;
mod player;
mod pos;
mod realm;
mod terrain;
mod weighted_dist;
mod world_data;
use bevy::prelude::*;
use draw2d::Draw2d;
use leafwing_input_manager::plugin::InputManagerPlugin;
use terrain::Terrain;
use world_data::WorldData;

struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldData::new())
            .add_startup_system(player::spawn_player)
            .add_system(player::move_player)
            .add_system(load_area::update_load_area)
            .add_system(load_area::load_order);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<player::Action>::default())
        .add_plugin(GameLogic)
        .run();
}
