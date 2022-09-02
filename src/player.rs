use crate::{load_area::LoadArea, pos::Pos};
use bevy::{
    math::Vec3,
    prelude::{Commands, KeyCode, Query, Res, Vec2},
    time::Time,
};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Dir {
    Up,
    Left,
    Down,
    Right,
}

impl From<Dir> for Vec2 {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::Up => Vec2::new(0., 1.),
            Dir::Left => Vec2::new(-1., 0.),
            Dir::Down => Vec2::new(0., -1.),
            Dir::Right => Vec2::new(1., 0.),
            _ => Vec2::default(),
        }
    }
}

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle((
            Pos::default(),
            LoadArea {
                realm: crate::realm::Realm::Earth,
                col: (0, 0),
                dist: 3,
            },
        ))
        .insert_bundle(InputManagerBundle::<Dir> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::W, Dir::Up),
                (KeyCode::A, Dir::Left),
                (KeyCode::S, Dir::Down),
                (KeyCode::D, Dir::Right),
                (KeyCode::Z, Dir::Up),
                (KeyCode::Q, Dir::Left),
            ]),
        });
}

pub fn move_player(mut query: Query<(&mut Pos, &ActionState<Dir>)>, time: Res<Time>) {
    let (mut pos, action_state) = query.single_mut();
    let mut movement = Vec2::default();
    for action in action_state.get_pressed() {
        movement += Vec2::from(action);
    }
    if movement.length_squared() > 0. {
        movement = movement.normalize() * 30. * time.delta_seconds();
        **pos = **pos + Vec3::new(movement.x, 0., movement.y);
    }
}
