use super::{
    Crouching, FreeFly, Speed, SteppingOn, Walking, block_action::BlockActionPlugin,
    key_binds::KeyBinds,
};
use crate::world::{BlockRayCastHit, Realm};
use crate::{
    Block,
    agents::{BodyExtents, Grounded, Heading, Jumping},
    items::{InventoryTrait, Item, Stack, new_inventory},
    sounds::{BlockSoundCD, FootstepCD, on_item_get},
    ui::{CursorGrabbed, ItemHolder},
};
use avian3d::prelude::{
    CoefficientCombine, Collider, Friction, LinearVelocity, LockedAxes, RigidBody, ShapeCaster,
};
use bevy::{math::Vec3, prelude::*};
use leafwing_input_manager::prelude::*;
use std::time::Duration;

/// Half-extents of the player's bounding box. Width/depth equal the capsule
/// radius; height equals half the total capsule height (cylinder + caps).
pub const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.25, 0.85, 0.25);
// Avian's `Collider::capsule(radius, length)` puts hemispheres on top of
// `length`, so cylinder length is total height minus two radii.
const PLAYER_CAPSULE_LENGTH: f32 = PLAYER_HALF_EXTENTS.y * 2. - PLAYER_HALF_EXTENTS.x * 2.;

const WALK_SPEED: f32 = 7.;
const FREE_FLY_X_SPEED: f32 = 500.;
const FREE_FLY_ACC_MULT: f32 = 150.;
const SPAWN: Vec3 = Vec3 {
    x: 280.,
    y: 500.,
    z: -150.,
};
pub const HOTBAR_SLOTS: usize = 8;

pub struct PlayerPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct PlayerSpawn;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(confy::load_path::<KeyBinds>("key_bindings.toml").unwrap())
            .add_plugins(BlockActionPlugin)
            .add_plugins(InputManagerPlugin::<Dir>::default())
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_plugins(InputManagerPlugin::<DevCommand>::default())
            .add_systems(
                Startup,
                (spawn_player, ApplyDeferred).chain().in_set(PlayerSpawn),
            )
            .add_systems(
                Update,
                (toggle_fly, ramp_up_freeflight_speed, move_player)
                    .chain()
                    .run_if(in_state(CursorGrabbed)),
            )
            .add_systems(OnExit(CursorGrabbed), reset_heading);
    }
}

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
    Hit,
    Modify,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum DevCommand {
    ToggleFly,
}

pub fn spawn_player(mut commands: Commands, key_binds: Res<KeyBinds>) {
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
            RigidBody::Dynamic,
            // Capsule's rounded base slides cleanly across coplanar triangle
            // edges in the chunk trimesh; a flat-bottom cuboid catches on them.
            Collider::capsule(PLAYER_HALF_EXTENTS.x, PLAYER_CAPSULE_LENGTH),
            // Material friction lives on the player and is rewritten each frame
            // from `SteppingOn`. `Min` combine ensures whichever of (player,
            // terrain) is slipperier wins, so ice etc. dominate.
            Friction::new(1.0).with_combine_rule(CoefficientCombine::Min),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            ShapeCaster::new(
                Collider::cuboid(
                    PLAYER_HALF_EXTENTS.x * 1.9,
                    0.05,
                    PLAYER_HALF_EXTENTS.z * 1.9,
                ),
                Vec3::new(0., -PLAYER_HALF_EXTENTS.y + 0.05, 0.),
                Quat::IDENTITY,
                Dir3::NEG_Y,
            )
            .with_max_distance(0.15),
            Grounded(false),
            BodyExtents(PLAYER_HALF_EXTENTS),
        ))
        .insert((
            Heading(Vec3::default()),
            Speed(WALK_SPEED),
            Jumping {
                force: 13.,
                cd: Timer::new(Duration::from_millis(500), TimerMode::Once),
                intent: false,
            },
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
        .observe(on_item_get);
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
