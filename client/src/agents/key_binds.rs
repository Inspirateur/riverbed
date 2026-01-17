use bevy::prelude::{KeyCode, MouseButton};
use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;

#[derive(Serialize, Deserialize, Resource)]
pub struct KeyBinds {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
    pub hit: MouseButton,
    pub modify: MouseButton,
    pub toggle_fly: KeyCode,
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            jump: KeyCode::Space,
            crouch: KeyCode::ShiftLeft,
            hit: MouseButton::Left,
            modify: MouseButton::Right,
            toggle_fly: KeyCode::F1,
        }
    }
}