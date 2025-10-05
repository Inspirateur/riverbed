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
use bevy::{prelude::*, window::{CursorGrabMode, CursorIcon, CursorOptions, SystemCursorIcon}};
use in_hand::InHandPlugin;
use leafwing_input_manager::prelude::*;
use ui_tex_map::UiTexMapPlugin;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameUiState>()
            .add_computed_state::<ScrollGrabbed>()
            .add_computed_state::<CursorGrabbed>()
            .add_computed_state::<Inventory>()
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
    /// Game states that need the cursor to operate in
    pub fn needs_free_cursor(&self) -> bool {
        matches!(self, GameUiState::InGameMenu | GameUiState::FurnaceMenu)
    }

    /// Game states that only need scrolling to operate in (includes all free cursor state)
    pub fn needs_scrolling(&self) -> bool {
        self.needs_free_cursor() || matches!(self, GameUiState::CraftingMenu)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScrollGrabbed;

impl ComputedStates for ScrollGrabbed {
    type SourceStates = GameUiState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        if sources.needs_scrolling() {
            None
        } else {
            Some(Self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inventory;

impl ComputedStates for Inventory {
    type SourceStates = GameUiState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        if sources.needs_free_cursor() && sources != GameUiState::InGameMenu {
            Some(Self)
        } else {
            None
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
    commands.spawn(input_map);
}

fn process_ui_actions(
    mut ui_action_query: Query<&ActionState<UIAction>>,
    game_ui_state: Res<State<GameUiState>>,
    mut next_ui_state: ResMut<NextState<GameUiState>>
) {
    let action_state = ui_action_query.single_mut().unwrap();
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
    mut cursor_options: Query<&mut CursorOptions>,
) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Confined;
}

fn free_cursor(
    mut cursor_options: Query<&mut CursorOptions>,
) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };
    cursor_options.visible = true;
    cursor_options.grab_mode = CursorGrabMode::None;
}

fn highlight_hover(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
    mut cursor: Query<&mut CursorIcon>
) {
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
    let Ok(mut cursor_icon) = cursor.single_mut() else {
        return;
    };
    *cursor_icon = CursorIcon::System(
        if hovering {
            SystemCursorIcon::Pointer
        } else {
            SystemCursorIcon::Default
        }
    );
}
