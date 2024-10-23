mod ui_tex_map;
mod game_menu;
mod debug_display;
mod hotbar;
mod craft_menu;
mod crosshair;
mod in_hand;
mod furnace_menu;
mod item_slots;
pub use item_slots::*;
use craft_menu::CraftMenuPlugin;
use furnace_menu::FurnaceMenuPlugin;
pub use furnace_menu::OpenFurnace;
use crosshair::setup_crosshair;
pub use hotbar::SelectedHotbarSlot;
use debug_display::DebugDisplayPlugin;
use game_menu::MenuPlugin;
use hotbar::HotbarPlugin;
use bevy::{prelude::*, window::CursorGrabMode};
use in_hand::InHandPlugin;
use leafwing_input_manager::prelude::*;
use ui_tex_map::UiTexMapPlugin;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameUiState>()
            .add_computed_state::<ControllingPlayer>()
            .add_computed_state::<CursorGrabbed>()
            .add_plugins(InputManagerPlugin::<UIAction>::default())
            .add_plugins(ItemSlotPlugin)
            .add_plugins(UiTexMapPlugin)
            .add_plugins(HotbarPlugin)
            .add_plugins(DebugDisplayPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(CraftMenuPlugin)
            .add_plugins(InHandPlugin)
            .add_plugins(FurnaceMenuPlugin)
            .add_systems(Startup, setup_ui_actions)
            .add_systems(Startup, setup_crosshair)
            .add_systems(Update, process_ui_actions)
            .add_systems(OnEnter(CursorGrabbed), grab_cursor)
            .add_systems(OnExit(CursorGrabbed), free_cursor)
            .add_systems(Update, highlight_hover.run_if(not(in_state(CursorGrabbed))))
            ;
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameUiState {
    #[default]
    None,
    InGameMenu,
    CraftingMenu,
    FurnaceMenu
}

impl GameUiState {
    pub fn needs_free_cursor(&self) -> bool {
        matches!(self, GameUiState::InGameMenu | GameUiState::FurnaceMenu)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControllingPlayer;

impl ComputedStates for ControllingPlayer {
    type SourceStates = GameUiState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        // For now ControllingPlayer == CursorGrabbed, but conceptually we could be CursorGrabbed and not controlling player
        if sources.needs_free_cursor() {
            None
        } else {
            Some(Self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CursorGrabbed;

impl ComputedStates for CursorGrabbed {
    type SourceStates = GameUiState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        if sources.needs_free_cursor() {
            None
        } else {
            Some(Self)
        }
    }
}

#[derive(Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum UIAction {
    Escape,
    CraftingMenu,
    ScrollUp, 
    ScrollDown,
}

fn setup_ui_actions(mut commands: Commands) {
    let mut input_map = InputMap::new([
        (UIAction::Escape, KeyCode::Escape),
        (UIAction::CraftingMenu, KeyCode::KeyC),
    ]);
    input_map.insert(UIAction::ScrollUp, MouseScrollDirection::UP);
    input_map.insert(UIAction::ScrollDown, MouseScrollDirection::DOWN);
    commands.spawn(InputManagerBundle::<UIAction> {
        action_state: ActionState::default(),
        input_map,
    });
}

fn process_ui_actions(
    mut ui_action_query: Query<&ActionState<UIAction>>,
    game_ui_state: Res<State<GameUiState>>,
    mut next_ui_state: ResMut<NextState<GameUiState>>
) {
    let action_state = ui_action_query.single_mut();
    for action in action_state.get_just_pressed() {
        if action == UIAction::Escape {
            if **game_ui_state == GameUiState::None {
                next_ui_state.set(GameUiState::InGameMenu);
            } else {
                next_ui_state.set(GameUiState::None);
            }
        } else if action == UIAction::CraftingMenu && **game_ui_state != GameUiState::InGameMenu {
            if **game_ui_state == GameUiState::None {
                next_ui_state.set(GameUiState::CraftingMenu);
            } else {
                next_ui_state.set(GameUiState::None);
            }
        }
    }
}

fn grab_cursor(
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Confined;
}

fn free_cursor(
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    window.cursor.visible = true;
    window.cursor.grab_mode = CursorGrabMode::None;
}

fn highlight_hover(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    let mut any_interaction = false;
    let mut hovering= false;
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        bg_color.0 = match interaction {
            Interaction::Hovered | Interaction::Pressed => {
                hovering = true;
                Color::linear_rgba(0., 0., 0., 0.6)
            },
            _ => Color::linear_rgba(0., 0., 0., 0.0),
        };
        any_interaction = true;
    }
    if !any_interaction {
        return;
    }
    window.cursor.icon = if hovering {
        CursorIcon::Pointer
    } else {
        CursorIcon::Default
    };
}