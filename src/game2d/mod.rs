mod draw2d;
mod render2d;
use bevy::prelude::Plugin;
use self::draw2d::Draw2d;

pub struct Game2d;

impl Plugin for Game2d {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(Draw2d);
    }
}