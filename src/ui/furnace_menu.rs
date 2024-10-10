use bevy::prelude::*;

use crate::{agents::PlayerControlled, items::{FiringTable, Hotbar}};

use super::{game_menu::despawn_screen, hotbar::UiTextureMap, GameUiState};

pub struct FurnaceMenuPlugin;

impl Plugin for FurnaceMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(OpenFurnace(None))
            .add_systems(OnEnter(GameUiState::FurnaceMenu), open_furnace_menu)
            .add_systems(OnExit(GameUiState::FurnaceMenu), despawn_screen::<FurnaceMenu>)
            //.add_systems(Update, ().chain().run_if(in_state(GameUiState::FurnaceMenu)))
            ;
    }
}

#[derive(Resource)]
pub struct OpenFurnace(pub Option<Entity>);

#[derive(Component)]
struct FurnaceMenu;

fn open_furnace_menu(
    commands: Commands,
    firing_table: Res<FiringTable>,
    hotbar_query: Query<&Hotbar, With<PlayerControlled>>,
    tex_map: Res<UiTextureMap>,
) {

}