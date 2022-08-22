use bevy::{
    math::Vec3,
    prelude::{Commands, Component, Query, With},
};

use crate::load_area::LoadArea;

#[derive(Component)]
pub struct Pos(pub Vec3);

#[derive(Component)]
pub struct Player;

pub fn spawn_player(mut commands: Commands) {
    commands.spawn_bundle((
        Pos(Vec3::default()),
        Player,
        LoadArea {
            realm: crate::realm::Realm::Earth,
            col: (0, 0),
            dist: 6,
        },
    ));
}

pub fn move_player(mut query: Query<&mut Pos, With<Player>>) {
    // Code inputs
    for mut pos in query.iter_mut() {
        pos.0.x += 0.01;
    }
}
