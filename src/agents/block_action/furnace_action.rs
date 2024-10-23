use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::{agents::{Action, PlayerControlled, TargetBlock}, ui::{furnace_slots, GameUiState, ItemHolder, OpenFurnace}, world::{BlockEntities, VoxelWorld}};

use super::block_hit_place::BlockAttached;

pub struct FurnaceActionPlugin;

impl Plugin for FurnaceActionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, open_furnace_menu.run_if(in_state(GameUiState::None)))
        ;
    }
}


#[derive(Debug, Component)]
// Add #[require(ItemHolder(furnace_slots))] when bevy 0.15 lands
pub struct Furnace {
    pub name: String,
    pub temp: u32,
}

impl Furnace {
    pub fn new(name: String, temp: u32) -> Self {
        Self {
            name,
            temp,
        }
    }
}

fn open_furnace_menu(
    mut commands: Commands,
    world: Res<VoxelWorld>, 
    block_action_query: Query<(&TargetBlock, &ActionState<Action>), With<PlayerControlled>>,
    furnace_query: Query<(&Furnace, &BlockAttached)>,
    mut block_entities: ResMut<BlockEntities>,
    mut next_ui_state: ResMut<NextState<GameUiState>>,
    mut furnace_menu: ResMut<OpenFurnace>
) {
    for (target_block_opt, action) in block_action_query.iter() {
        if !action.just_pressed(&Action::Modify) {
            continue;
        }
        let Some(target_block) = &target_block_opt.0 else {
            continue;
        };
        let furnace = world.get_block(target_block.pos);
        let Some(furnace_temp) = furnace.furnace_temp() else {
            continue;
        };
        let furnace_ent = if let Some(ent) = block_entities.get(&target_block.pos) {
            if furnace_query.contains(ent) {
                ent
            } else {
                commands.entity(ent)
                    .insert(Furnace::new(furnace.to_string(), furnace_temp))
                    .insert(furnace_slots())
                    .id()
            }
        } else {
            let ent = commands
                .spawn(Furnace::new(furnace.to_string(), furnace_temp))
                .insert(furnace_slots())
                .id();
            block_entities.add(&target_block.pos, ent);
            ent
        };
        furnace_menu.0 = Some(furnace_ent);
        next_ui_state.set(GameUiState::FurnaceMenu);
    }
}
