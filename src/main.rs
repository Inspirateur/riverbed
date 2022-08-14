mod draw2d;
mod draw3d;
mod noise_utils;
mod terrain;
mod weighted_dist;
use bevy::prelude::*;
use draw2d::Draw2d;
use terrain::Terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Terrain)
        .add_plugin(Draw2d)
        .run();
}
