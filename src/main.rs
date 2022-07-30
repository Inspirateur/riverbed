mod conditionned_index;
mod draw2d;
mod draw3d;
mod noise_utils;
mod terrain;
use bevy::prelude::*;
use draw3d::Draw3d;
use terrain::Terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Terrain)
        .add_plugin(Draw3d)
        .run();
}
