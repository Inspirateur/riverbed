mod bloc;
mod chunk;
mod draw2d;
mod load_area;
mod noise_utils;
mod packed_ints;
mod player;
mod realm;
mod terrain;
mod weighted_dist;
use bevy::prelude::*;
use draw2d::Draw2d;
use terrain::Terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(player::spawn_player.system())
        .add_system(player::move_player.system())
        .add_system(load_area::update_load_area.system())
        .add_system(load_area::load_order.system())
        .run();
}
