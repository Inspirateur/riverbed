use super::{block_action::BlockActionPlugin, key_binds::KeyBinds};
use crate::{
    game_state::CursorGrabbed,
    sound_components::{BlockSoundCD, FootstepCD},
};
use bevy::{
    math::Vec3,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use leafwing_input_manager::prelude::*;
use rb_block::Block;
use rb_camera::{CameraSpawn, FpsCam, PlayerControlled};
use rb_items::{InventoryTrait, Item, ItemHolder, Stack, new_inventory};
use rb_physics::*;
use rb_world::{BlockRayCastHit, Realm};
use std::{f32::consts::FRAC_PI_2, time::Duration};

const WALK_SPEED: f32 = 7.;
const FREE_FLY_X_SPEED: f32 = 500.;
const FREE_FLY_ACC_MULT: f32 = 150.;
const SPAWN: Vec3 = Vec3 {
    x: 280.,
    y: 500.,
    z: -150.,
};
const CAMERA_PAN_RATE: f32 = 0.001;
pub const HOTBAR_SLOTS: usize = 8;

pub struct PlayerPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct PlayerSpawn;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(confy::load_path::<KeyBinds>("key_bindings.toml").unwrap())
            .add_plugins(BlockActionPlugin)
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_plugins(InputManagerPlugin::<Dir>::default())
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_plugins(InputManagerPlugin::<DevCommand>::default())
            .add_systems(
                Startup,
                (spawn_player, ApplyDeferred)
                    .chain()
                    .in_set(PlayerSpawn)
                    .in_set(CameraSpawn),
            )
            .add_systems(
                Update,
                (toggle_fly, ramp_up_freeflight_speed, move_player)
                    .chain()
                    .run_if(in_state(CursorGrabbed)),
            )
            .add_systems(Update, pan_camera.run_if(in_state(CursorGrabbed)))
            .add_systems(OnExit(CursorGrabbed), reset_heading);
    }
}

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
    Hit,
    Modify,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum DevCommand {
    ToggleFly,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
pub enum CameraMovement {
    Pan,
}

impl Actionlike for CameraMovement {
    fn input_control_kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }
}

pub fn spawn_player(
    mut commands: Commands,
    key_binds: Res<KeyBinds>,
    mut cursor_options: Query<&mut CursorOptions>,
) {
    let input_map = InputMap::default().with_dual_axis(CameraMovement::Pan, MouseMove::default());
    let aabb = AABB(Vec3::new(0.5, 1.7, 0.5));
    let cam = commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(aabb.0.x / 2., aabb.0.y - 0.05, aabb.0.z / 2.).looking_at(
                Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 1.,
                },
                Vec3::Y,
            ),
            Projection::Perspective(PerspectiveProjection {
                far: 10000.,
                fov: FRAC_PI_2 * 9. / 16.,
                ..Default::default()
            }),
        ))
        .insert(Msaa::default())
        .insert(input_map)
        .insert(FpsCam::default())
        .id();

    let realm = Realm::Overworld;
    let mut inventory = new_inventory::<HOTBAR_SLOTS>();
    inventory.try_add(Stack::Some(Item::Block(Block::Smelter), 1));
    inventory.try_add(Stack::Some(Item::Coal, 20));
    inventory.try_add(Stack::Some(Item::IronOre, 50));
    commands
        .spawn((
            Transform {
                translation: SPAWN,
                ..default()
            },
            Visibility::default(),
            realm,
            Gravity(50.),
            Heading(Vec3::default()),
            Speed(WALK_SPEED),
            Jumping {
                force: 13.,
                cd: Timer::new(Duration::from_millis(500), TimerMode::Once),
                intent: false,
            },
            aabb,
            Velocity(Vec3::default()),
            TargetBlock(None),
            ItemHolder::Inventory(inventory),
            PlayerControlled,
        ))
        .insert((Walking, SteppingOn(Block::Air), Crouching(false)))
        .insert(SpatialListener::new(0.3))
        .insert((FootstepCD(0.), BlockSoundCD(0.)))
        .insert(InputMap::new([
            (Dir::Front, key_binds.forward),
            (Dir::Left, key_binds.left),
            (Dir::Back, key_binds.backward),
            (Dir::Right, key_binds.right),
            (Dir::Down, key_binds.crouch),
            (Dir::Up, key_binds.jump),
        ]))
        .insert(InputMap::new([
            (Action::Hit, key_binds.hit),
            (Action::Modify, key_binds.modify),
        ]))
        .insert(InputMap::new([(
            DevCommand::ToggleFly,
            key_binds.toggle_fly,
        )]))
        .add_child(cam);

    let mut cursor_options = cursor_options.single_mut().unwrap();
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

pub fn move_player(
    mut player_query: Query<(
        &mut Heading,
        &mut Jumping,
        &mut Crouching,
        &Speed,
        &ActionState<Dir>,
    )>,
    cam_query: Query<&Transform, With<Camera>>,
) {
    let cam_transform = if let Ok(ct) = cam_query.single() {
        *ct
    } else {
        Transform::default()
    };
    let (mut heading, mut jumping, mut crouching, speed, action_state) =
        player_query.single_mut().unwrap();
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
        movement = Vec3::Y.cross(*cam_transform.right()) * movement.z
            + cam_transform.right() * movement.x
            + movement.y * Vec3::Y;
    }
    heading.0 = movement * speed.0;
    heading.0.y = f32::NAN;
}

fn toggle_fly(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut Speed,
        &ActionState<DevCommand>,
        Option<&Walking>,
    )>,
) {
    let (entity, mut speed, action_state, walking_opt) = player_query.single_mut().unwrap();
    for dev_command in action_state.get_just_pressed() {
        if dev_command == DevCommand::ToggleFly {
            if walking_opt.is_some() {
                commands.entity(entity).remove::<Walking>().insert(FreeFly);
            } else {
                commands.entity(entity).remove::<FreeFly>().insert(Walking);
                speed.0 = WALK_SPEED;
            }
        }
    }
}

fn ramp_up_freeflight_speed(
    time: Res<Time>,
    mut player_query: Query<(&mut Speed, &Heading), With<FreeFly>>,
) {
    let Ok((mut speed, heading)) = player_query.single_mut() else {
        return;
    };
    if heading.0.x.abs() + heading.0.z.abs() > 0. {
        speed.0 = (speed.0 + time.delta_secs() * FREE_FLY_ACC_MULT).min(FREE_FLY_X_SPEED);
    } else {
        speed.0 = WALK_SPEED;
    }
}

fn reset_heading(mut player_query: Query<&mut Heading, With<PlayerControlled>>) {
    let Ok(mut heading) = player_query.single_mut() else {
        return;
    };
    heading.0 = Vec3::new(0., 0., 0.);
}

fn pan_camera(mut query: Query<(&ActionState<CameraMovement>, &mut FpsCam)>) {
    let (action_state, mut fpscam) = query.single_mut().unwrap();
    let camera_pan_vector = action_state.axis_pair(&CameraMovement::Pan);
    fpscam.yaw -= CAMERA_PAN_RATE * camera_pan_vector.x;
    fpscam.pitch -= CAMERA_PAN_RATE * camera_pan_vector.y;
    fpscam.pitch = fpscam.pitch.clamp(-1.5, 1.5);
}
