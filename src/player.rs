use crate::load_area::LoadArea;
use ourcraft::{Pos, ChunkPos2D, Realm};
use bevy::{
    math::Vec3,
    prelude::{Commands, KeyCode, Query, Res},
    time::Time, reflect::TypePath,
};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, TypePath, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Dir {
    Front,
    Back,
    Up,
    Left,
    Down,
    Right,
}

impl From<Dir> for Vec3 {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::Front => Vec3::new(0., 0., 1.),
            Dir::Back => Vec3::new(0., 0., -1.),
            Dir::Up => Vec3::new(0., 1., 0.),
            Dir::Down => Vec3::new(0., -1., 0.),
            Dir::Right => Vec3::new(1., 0., 0.),
            Dir::Left => Vec3::new(-1., 0., 0.),
        }
    }
}

pub fn spawn_player(mut commands: Commands) {
    let spawn = Pos::<f32> {
        realm: Realm::Overworld,
        x: 0.,
        y: 0.,
        z: 0.,
    };
    commands
        .spawn((
            spawn,
            LoadArea {
                col: ChunkPos2D::from(spawn),
                dist: 5,
            },
        ))
        .insert(InputManagerBundle::<Dir> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::W, Dir::Front),
                (KeyCode::A, Dir::Left),
                (KeyCode::S, Dir::Back),
                (KeyCode::D, Dir::Right),
                (KeyCode::Z, Dir::Front),
                (KeyCode::Q, Dir::Left),
            ]),
        });
}

pub fn move_player(mut query: Query<(&mut Pos, &ActionState<Dir>)>, time: Res<Time>) {
    let (mut pos, action_state) = query.single_mut();
    let mut movement = Vec3::default();
    for action in action_state.get_pressed() {
        movement += Vec3::from(action);
    }
    if movement.length_squared() > 0. {
        movement = movement.normalize() * 40. * time.delta_seconds();
        *pos += movement;
    }
}
