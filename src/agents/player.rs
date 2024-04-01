use std::time::Duration;
use crate::{gen::RenderDistance, agents::{Gravity, Heading, AABB, Velocity, Jumping}, GameState};
use crate::blocks::{Realm, BlockRayCastHit};
use bevy::{
    math::Vec3,
    prelude::*,
};
use leafwing_input_manager::prelude::*;
const SPAWN: Vec3 = Vec3 { x: 540., y: 500., z: 130.};

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct TargetBlock(pub Option<BlockRayCastHit>);

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum Action {
    Action1,
    Action2,
}

#[derive(Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum UIAction {
    Escape,
    Inventory,
}

pub fn spawn_player(mut commands: Commands) {    
    let realm = Realm::Overworld;
    let transform = SpatialBundle {
        transform: Transform {translation: SPAWN, ..default()},
        ..default()
    };
    let rd = RenderDistance(32);
    commands
        .spawn((
            transform,
            realm,
            Gravity(3.),
            Heading(Vec3::default()),
            Jumping {force: 1., cd: Timer::new(Duration::from_millis(500), TimerMode::Once), intent: false},
            AABB(Vec3::new(0.5, 1.7, 0.5)),
            Velocity(Vec3::default()),
            rd,
            TargetBlock(None),
            PlayerControlled
        ))
        .insert(InputManagerBundle::<Dir> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (Dir::Front, KeyCode::KeyW),
                (Dir::Left, KeyCode::KeyA),
                (Dir::Back, KeyCode::KeyS),
                (Dir::Right, KeyCode::KeyD),
                (Dir::Front, KeyCode::KeyZ),
                (Dir::Left, KeyCode::KeyQ),
                (Dir::Down, KeyCode::ShiftLeft),
                (Dir::Up, KeyCode::Space)
            ]),
        })
        .insert(InputManagerBundle::<UIAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (UIAction::Escape, KeyCode::Escape),
            ]),
        })
        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (Action::Action1, MouseButton::Left),
                (Action::Action2, MouseButton::Right),
            ])
        });
}

pub fn move_player(
    mut player_query: Query<(&mut Heading, &mut Jumping, &ActionState<Dir>)>, 
    cam_query: Query<&Transform, With<Camera>>, 
) {
    let cam_transform = if let Ok(ct) = cam_query.get_single() {
        *ct
    } else {
        Transform::default()
    };
    let (mut heading, mut jumping, action_state) = player_query.single_mut();
    jumping.intent = false;
    let mut movement = Vec3::default();
    for action in action_state.get_pressed() {
        movement += Vec3::from(action);
    }
    if movement.length_squared() > 0. {
        if movement.y > 0. {
            jumping.intent = true;
        }
        movement = movement.normalize();
        movement = Vec3::Y.cross(*cam_transform.right())*movement.z + cam_transform.right()*movement.x + movement.y * Vec3::Y;
    }
    heading.0 = movement;
    heading.0.y = f32::NAN;
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct PlayerSpawn;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(InputManagerPlugin::<Dir>::default())
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_plugins(InputManagerPlugin::<UIAction>::default())
            .add_systems(Startup, (spawn_player, apply_deferred).chain().in_set(PlayerSpawn))
            .add_systems(Update, move_player.run_if(in_state(GameState::Game)))
        ;
    }
}