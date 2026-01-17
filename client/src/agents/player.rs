use crate::{
    agents::{Gravity, Heading, Jumping, Velocity, AABB},
    sounds::{on_item_get, BlockSoundCD, FootstepCD},
    ui::CursorGrabbed,
};
use bevy::{math::Vec3, prelude::*};
use leafwing_input_manager::prelude::*;
use shared::world::pos::pos2d::ColPos;
use shared::world::pos::PlayerCol;
use shared::{
    block::Block,
    items::{item_slots::ItemHolder, new_inventory, InventoryTrait, Item, Stack},
    world::{realm::Realm, BlockRayCastHit},
};
use shared::{DEFAULT_SPAWN_POSITION, PLAYER_AABB, PLAYER_GRAVITY, PLAYER_JUMP_FORCE, WALK_SPEED};
use std::time::Duration;

use super::{
    block_action::BlockActionPlugin, key_binds::KeyBinds, Crouching, FreeFly, Speed, SteppingOn,
    Walking,
};
pub const HOTBAR_SLOTS: usize = 8;

pub struct PlayerPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct PlayerSpawn;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(confy::load_path::<KeyBinds>("key_bindings.toml").unwrap())
            .add_plugins(BlockActionPlugin)
            // Action (Hit/Modify) is used for block interactions
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_systems(
                Startup,
                (spawn_player, ApplyDeferred).chain().in_set(PlayerSpawn),
            )
            .add_systems(Update, update_player_col);
    }
}

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct TargetBlock(pub Option<BlockRayCastHit>);

/// Actions for block interaction (hit/break, modify/place)
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug, Hash, Reflect)]
pub enum Action {
    Hit,
    Modify,
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
                translation: DEFAULT_SPAWN_POSITION,
                ..default()
            },
            Visibility::default(),
            realm,
            Gravity(PLAYER_GRAVITY),
            Heading(Vec3::default()),
            Speed(WALK_SPEED),
            Jumping {
                force: PLAYER_JUMP_FORCE,
                cd: Timer::new(Duration::from_millis(500), TimerMode::Once),
                intent: false,
            },
            AABB(PLAYER_AABB),
            Velocity(Vec3::default()),
            TargetBlock(None),
            ItemHolder::Inventory(inventory),
            PlayerControlled,
        ))
        .insert((Walking, SteppingOn(Block::Air), Crouching(false)))
        .insert(SpatialListener::new(0.3))
        .insert((FootstepCD(0.), BlockSoundCD(0.)))
        .insert(InputMap::new([
            (Action::Hit, key_binds.hit),
            (Action::Modify, key_binds.modify),
        ]))
        .observe(on_item_get);
}

/// Updates the PlayerCol component when the player moves to a different chunk column.
/// This system ensures LOD remeshing is triggered when the player's position changes.
fn update_player_col(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &Transform, &Realm, Option<&mut PlayerCol>),
        With<PlayerControlled>,
    >,
) {
    let Ok((entity, transform, realm, player_col_opt)) = player_query.single_mut() else {
        return;
    };

    let new_col = ColPos::from((transform.translation, *realm));

    match player_col_opt {
        Some(mut player_col) => {
            // Update only if the column changed
            if player_col.0 != new_col {
                player_col.0 = new_col;
            }
        }
        None => {
            // Initial assignment
            commands.entity(entity).insert(PlayerCol(new_col));
        }
    }
}
