use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::{ClientToServerMessage};

use crate::agents::key_binds::KeyBinds;
use crate::agents::PlayerControlled;
use crate::agents::Velocity;
use crate::render::FpsCam;
use crate::ui::SelectedHotbarSlot;
use crate::network::TargetServerState;

use super::buffered_client::{CurrentFrameInputs, CurrentFrameInputsExt, SyncTime, SyncTimeExt};
use shared::net::input_history::InputHistory;
use super::SendGameMessageExtension;

pub fn pre_input_update_system(
    mut frame_inputs: ResMut<CurrentFrameInputs>,
    mut input_history: ResMut<InputHistory>,
    mut sync_time: ResMut<SyncTime>,
) {
    sync_time.advance();

    let inputs_of_last_frame = frame_inputs.0.clone();
    input_history.push_frame(inputs_of_last_frame);
    
    frame_inputs.reset(sync_time.now_synced() as u64, sync_time.delta());
}

pub fn capture_player_inputs_system(
    key_binds: Res<KeyBinds>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut frame_inputs: ResMut<CurrentFrameInputs>,
) {
    if frame_inputs.0.delta_ms == 0 {
        return;
    }

    for keycode in keyboard.get_pressed() {
        if let Some(action) = key_binds.keycode_to_action(*keycode) {
            frame_inputs.0.inputs.insert(action);
        }
    }

    for button in mouse.get_pressed() {
        if let Some(action) = key_binds.mousebutton_to_action(*button) {
            frame_inputs.0.inputs.insert(action);
        }
    }

    for keycode in keyboard.get_just_pressed() {
        if let Some(action) = key_binds.keycode_to_action(*keycode) {
            frame_inputs.0.inputs.insert(action);
        }
    }
}

pub fn update_frame_inputs_system(
    camera: Query<&Transform, With<FpsCam>>,
    player: Query<(&Transform, &Velocity), (With<PlayerControlled>, Without<FpsCam>)>,
    selected_slot: Res<SelectedHotbarSlot>,
    mut frame_inputs: ResMut<CurrentFrameInputs>,
) {
    if frame_inputs.0.delta_ms == 0 {
        return;
    }

    if let Ok(camera_transform) = camera.single() {
        frame_inputs.0.camera = *camera_transform;
    }

    if let Ok((player_transform, velocity)) = player.single() {
        frame_inputs.0.position = player_transform.translation;
        frame_inputs.0.velocity = velocity.0;
    }

    frame_inputs.0.hotbar_slot = selected_slot.0 as u32;
}

pub fn upload_player_inputs_system(
    mut client: ResMut<RenetClient>,
    mut input_history: ResMut<InputHistory>,
    target: Res<crate::network::TargetServer>,
) {
    if client.is_disconnected() {
        input_history.clear_all();
        return;
    }

    if target.state != TargetServerState::FullyReady {
        return;
    }

    let frames = input_history.take_pending();
    if !frames.is_empty() {
        client.send_game_message(ClientToServerMessage::PlayerInputs(frames));
    }
}
