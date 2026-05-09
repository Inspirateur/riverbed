use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

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

    pub fn needs_scrolling(&self) -> bool {
        self.needs_free_cursor() || matches!(self, GameUiState::CraftingMenu)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CursorGrabbed;

impl ComputedStates for CursorGrabbed {
    type SourceStates = GameUiState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        if sources.needs_free_cursor() { None } else { Some(Self) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScrollGrabbed;

impl ComputedStates for ScrollGrabbed {
    type SourceStates = GameUiState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        if sources.needs_scrolling() { None } else { Some(Self) }
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

#[derive(Resource)]
pub struct SelectedHotbarSlot(pub usize);

#[derive(Resource)]
pub struct Dragging(pub rb_items::Stack);
