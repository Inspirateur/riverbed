use std::time::Duration;
use crate::{agents::{Gravity, Heading, Jumping, Velocity, AABB}, gen::RenderDistance, items::{Hotbar, Inventory}, GameState};
use crate::blocks::{Realm, BlockRayCastHit};
use bevy::{
    math::Vec3,
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use super::{block_action::BlockActionPlugin, Crouching, FreeFly, Speed, Walking};

const WALK_SPEED: f32 = 15.;
const FREE_FLY_X_SPEED: f32 = 80.;

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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum DevCommand {
    ToggleFly,
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
    let rd = RenderDistance(16);
    commands
        .spawn((
            transform,
            realm,
            Gravity(50.),
            Heading(Vec3::default()),
            Walking,
            Speed(WALK_SPEED),
            Jumping {force: 13., cd: Timer::new(Duration::from_millis(500), TimerMode::Once), intent: false},
            Crouching(false),
            AABB(Vec3::new(0.5, 1.7, 0.5)),
            Velocity(Vec3::default()),
            rd,
            TargetBlock(None),
            Hotbar(Inventory::new()),
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
        })
        .insert(InputManagerBundle::<DevCommand> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (DevCommand::ToggleFly, KeyCode::F1)
            ])
        });
}

pub fn move_player(
    mut player_query: Query<(&mut Heading, &mut Jumping, &mut Crouching, &Speed, &ActionState<Dir>)>, 
    cam_query: Query<&Transform, With<Camera>>, 
) {
    let cam_transform = if let Ok(ct) = cam_query.get_single() {
        *ct
    } else {
        Transform::default()
    };
    let (mut heading, mut jumping, mut crouching, speed, action_state) = player_query.single_mut();
    jumping.intent = false;
    crouching.0 = false;

    let mut movement = Vec3::default();
    for action in action_state.get_pressed() {
        if action == Dir::Up {
            jumping.intent = true;
        } else if action == Dir::Down {
            crouching.0 = true;
        } else {
            movement += Vec3::from(action);
        }
    }
    if movement.length_squared() > 0. {
        movement = movement.normalize();
        movement = Vec3::Y.cross(*cam_transform.right())*movement.z + cam_transform.right()*movement.x + movement.y * Vec3::Y;
    }
    heading.0 = movement*speed.0;
    heading.0.y = f32::NAN;
}

fn toggle_fly(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Speed, &ActionState<DevCommand>, Option<&Walking>)>
) {
    let (entity, mut speed, action_state, walking_opt) = player_query.single_mut();
    for dev_command in action_state.get_just_pressed() {
        if dev_command == DevCommand::ToggleFly {
            if walking_opt.is_some() {
                commands.entity(entity).remove::<Walking>().insert(FreeFly);
                speed.0 = FREE_FLY_X_SPEED;
            } else {
                commands.entity(entity).remove::<FreeFly>().insert(Walking);
                speed.0 = WALK_SPEED;
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct PlayerSpawn;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(BlockActionPlugin)
            .add_plugins(InputManagerPlugin::<Dir>::default())
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_plugins(InputManagerPlugin::<UIAction>::default())
            .add_plugins(InputManagerPlugin::<DevCommand>::default())
            .add_systems(Startup, (spawn_player, apply_deferred).chain().in_set(PlayerSpawn))
            .add_systems(Update, move_player.run_if(in_state(GameState::Game)))
            .add_systems(Update, toggle_fly.run_if(in_state(GameState::Game)))
        ;
    }
}