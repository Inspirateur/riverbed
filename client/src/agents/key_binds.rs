use bevy::prelude::{KeyCode, MouseButton};
use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;
use shared::messages::TransmittableAction;

#[derive(Serialize, Deserialize, Resource)]
pub struct KeyBinds {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
    pub toggle_fly: KeyCode,
    pub hit: MouseButton,
    pub modify: MouseButton,
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
            toggle_fly: KeyCode::F1,
            hit: MouseButton::Left,
            modify: MouseButton::Right,
        }
    }
}

impl KeyBinds {
    fn keycode_to_action(&self, keycode: KeyCode) -> Option<TransmittableAction> {
        match keycode {
            k if k == self.forward => Some(TransmittableAction::MoveForward),
            k if k == self.backward => Some(TransmittableAction::MoveBackward),
            k if k == self.left => Some(TransmittableAction::MoveLeft),
            k if k == self.right => Some(TransmittableAction::MoveRight),
            k if k == self.jump => Some(TransmittableAction::JumpOrFlyUp),
            k if k == self.crouch => Some(TransmittableAction::CrouchOrFlyDown),
            k if k == self.toggle_fly => Some(TransmittableAction::ToggleFlyMode),
            _ => None,
        }
    }

    fn mousebutton_to_action(&self, button: MouseButton) -> Option<TransmittableAction> {
        match button {
            b if b == self.hit => Some(TransmittableAction::Hit),
            b if b == self.modify => Some(TransmittableAction::Modify),
            _ => None,
        }
    }
}
