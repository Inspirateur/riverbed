mod bloc;
mod bloc_pos;
mod blocs;
mod chunk;
mod col_commands;
mod draw2d;
mod earth_gen;
mod load_area;
mod load_cols;
mod player;
mod pos;
mod realm;
mod terrain_gen;
mod utils;
use bevy::prelude::*;
use blocs::Blocs;
use col_commands::ColCommands;
use draw2d::Draw2d;
use leafwing_input_manager::plugin::InputManagerPlugin;
use load_cols::{ColLoadEvent, ColUnloadEvent};
use std::sync::Arc;
use terrain_gen::Generators;
use utils::get_set;
use utils::noise_op;
use utils::packed_ints;
use utils::weighted_dist;
use utils::noise_build;

struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColCommands::new())
            .insert_resource(Blocs::new())
            .insert_resource(Arc::new(Generators::new(0)))
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
        .insert_resource(WindowDescriptor {
            title: "OurCraft".to_string(),
            width: 512.,
            height: 512.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<player::Dir>::default())
        .add_plugin(GameLogic)
        .add_plugin(Draw2d)
        .run();
}
