mod game_menu;
mod debug_display;
mod hotbar;
mod craft_menu;
mod crosshair;
use craft_menu::CraftMenuPlugin;
use crosshair::setup_crosshair;
pub use hotbar::SelectedHotbarSlot;
use debug_display::DebugDisplayPlugin;
use game_menu::MenuPlugin;
use hotbar::HotbarPlugin;
use bevy::{prelude::*, window::CursorGrabMode};
use leafwing_input_manager::prelude::*;
use crate::GameState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameUiState>()
            .add_plugins(InputManagerPlugin::<UIAction>::default())
            .add_plugins(HotbarPlugin)
            .add_plugins(DebugDisplayPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(CraftMenuPlugin)
            .add_systems(Startup, setup_ui_actions)
            .add_systems(Startup, setup_crosshair)
            .add_systems(Update, process_ui_actions)
            .add_systems(Update, update_cursor_grab)
            ;
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameUiState {
    #[default]
    None,
    Inventory,
    CraftingMenu,
}

#[derive(Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum UIAction {
    Escape,
    CraftingMenu,
}

fn setup_ui_actions(mut commands: Commands) {
    commands.spawn(InputManagerBundle::<UIAction> {
        action_state: ActionState::default(),
        input_map: InputMap::new([
            (UIAction::Escape, KeyCode::Escape),
            (UIAction::CraftingMenu, KeyCode::KeyC)
        ]),
    });
}

fn process_ui_actions(
    mut ui_action_query: Query<&ActionState<UIAction>>,
    game_state: Res<State<GameState>>,
    game_ui_state: Res<State<GameUiState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_ui_state: ResMut<NextState<GameUiState>>
) {
    let action_state = ui_action_query.single_mut();
    for action in action_state.get_just_pressed() {
        if action == UIAction::Escape {
            if **game_state == GameState::Menu {
                next_game_state.set(GameState::Game);
            } else {
                next_game_state.set(GameState::Menu);
            }
        } else if action == UIAction::CraftingMenu {
            if **game_ui_state == GameUiState::CraftingMenu {
                next_ui_state.set(GameUiState::None);
            } else {
                next_ui_state.set(GameUiState::CraftingMenu);
            }
        }
    }
}

fn update_cursor_grab(
    mut windows: Query<&mut Window>,
    game_state: Res<State<GameState>>, 
    game_ui_state: Res<State<GameUiState>>
) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    if **game_state == GameState::Menu || **game_ui_state == GameUiState::Inventory {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    } else {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Confined;
    }
}