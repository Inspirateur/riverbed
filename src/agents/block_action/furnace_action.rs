use super::block_hit_place::BlockAttached;
use crate::{
    agents::{Action, PlayerControlled, TargetBlock},
    items::{FiringTable, LitFurnace, Stack},
    ui::{furnace_slots, GameUiState, ItemHolder, OpenFurnace},
    world::{BlockEntities, BlockPos, VoxelWorld},
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use std::fs;

pub struct FurnaceActionPlugin;

impl Plugin for FurnaceActionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(
            json5::from_str::<FiringTable>(
                &fs::read_to_string("assets/data/firing.json5").unwrap(),
            )
            .unwrap(),
        )
        .add_systems(
            Update,
            open_furnace_menu.run_if(in_state(GameUiState::None)),
        )
        .add_systems(Update, on_furnace_edit)
        .add_systems(Update, tick_furnaces);
    }
}

#[derive(Debug, Component)]
// Add #[require(ItemHolder(furnace_slots))] when bevy 0.15 lands
pub struct Furnace {
    pub name: String,
    pub temp: u32,
    pub block_pos: BlockPos,
}

fn open_furnace_menu(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    block_action_query: Query<(&TargetBlock, &ActionState<Action>), With<PlayerControlled>>,
    furnace_query: Query<&Furnace>,
    mut block_entities: ResMut<BlockEntities>,
    mut next_ui_state: ResMut<NextState<GameUiState>>,
    mut furnace_menu: ResMut<OpenFurnace>,
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
                let ent = commands
                    .entity(ent)
                    .insert(Furnace {
                        name: furnace.to_string(),
                        temp: furnace_temp,
                        block_pos: target_block.pos,
                    })
                    .insert(furnace_slots())
                    .id();
                block_entities.add(&target_block.pos, ent);
                ent
            }
        } else {
            let ent = commands
                .spawn(Furnace {
                    name: furnace.to_string(),
                    temp: furnace_temp,
                    block_pos: target_block.pos,
                })
                .insert(furnace_slots())
                .id();
            block_entities.add(&target_block.pos, ent);
            ent
        };
        furnace_menu.0 = Some(furnace_ent);
        next_ui_state.set(GameUiState::FurnaceMenu);
    }
}

fn on_furnace_edit(
    voxel_world: Res<VoxelWorld>,
    mut commands: Commands,
    item_holders: Query<(Entity, &ItemHolder, &Furnace, Option<&LitFurnace>), Changed<ItemHolder>>,
    firing_table: Res<FiringTable>,
) {
    for (furnace_entt, item_holder, furnace, lit_furnace_opt) in item_holders.iter() {
        let Some(mut new_lit_furnace) = firing_table.get(item_holder, furnace.temp) else {
            // Turn furnace off
            commands.entity(furnace_entt).remove::<LitFurnace>();
            voxel_world.set_block(
                furnace.block_pos,
                voxel_world.get_block(furnace.block_pos).off(),
            );
            continue;
        };
        // If firing continues we inherit the previous remaining fuel sec
        if let Some(lit_furnace) = lit_furnace_opt {
            if lit_furnace.fuel_sec > 0. {
                new_lit_furnace.fuel_sec = lit_furnace.fuel_sec;
            }
        }
        // Replace the previous value
        commands.entity(furnace_entt).insert(new_lit_furnace);
        voxel_world.set_block(
            furnace.block_pos,
            voxel_world.get_block(furnace.block_pos).on(),
        );
    }
}

fn tick_furnaces(mut item_holders: Query<(&mut ItemHolder, &mut LitFurnace)>, time: Res<Time>) {
    for (mut item_holder, mut lit_furnace) in item_holders.iter_mut() {
        if lit_furnace.fuel_sec <= 0. || lit_furnace.firing_sec <= 0. {
            continue;
        }
        lit_furnace.fuel_sec -= time.delta_secs();
        lit_furnace.firing_sec -= time.delta_secs();
        // Early return to avoid triggering change detection
        if lit_furnace.firing_sec > 0. && lit_furnace.fuel_sec > 0. {
            continue;
        }
        let ItemHolder::Furnace {
            fuel,
            material,
            output,
        } = item_holder.as_mut()
        else {
            continue;
        };
        if lit_furnace.firing_sec <= 0. {
            material.take(1);
            output.try_add(Stack::Some(lit_furnace.output, 1));
        }
        if lit_furnace.fuel_sec <= 0. {
            fuel.take(1);
        }
    }
}

