use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::{ClientToServerMessage, ClientPlayerInput};

use crate::agents::key_binds::KeyBinds;
use crate::agents::PlayerControlled;
use crate::render::FpsCam;
use crate::ui::SelectedHotbarSlot;

use super::buffered_client::{CurrentFrameInputs, CurrentFrameInputsExt, PlayerTickInputsBuffer, SyncTime, SyncTimeExt};
use super::SendGameMessageExtension;

#[derive(Debug, Default, Resource)]
pub struct UnacknowledgedInputs(pub Vec<ClientPlayerInput>);

pub fn pre_input_update_system(
    mut frame_inputs: ResMut<CurrentFrameInputs>,
    mut tick_buffer: ResMut<PlayerTickInputsBuffer>,
    mut sync_time: ResMut<SyncTime>,
) {
    sync_time.advance();

    let inputs_of_last_frame = frame_inputs.0.clone();
    tick_buffer.buffer.push(inputs_of_last_frame);
    
    frame_inputs.reset(sync_time.curr_time_ms, sync_time.delta());
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
    player: Query<&Transform, (With<PlayerControlled>, Without<FpsCam>)>,
    selected_slot: Res<SelectedHotbarSlot>,
    mut frame_inputs: ResMut<CurrentFrameInputs>,
) {
    if frame_inputs.0.delta_ms == 0 {
        return;
    }

    if let Ok(camera_transform) = camera.single() {
        frame_inputs.0.camera = *camera_transform;
    }

    if let Ok(player_transform) = player.single() {
        frame_inputs.0.position = player_transform.translation;
    }

    frame_inputs.0.hotbar_slot = selected_slot.0 as u32;
}

pub fn upload_player_inputs_system(
    mut client: ResMut<RenetClient>,
    mut inputs: ResMut<PlayerTickInputsBuffer>,
    mut unacknowledged_inputs: ResMut<UnacknowledgedInputs>,
) {
    if client.is_disconnected() {
        inputs.buffer.clear();
        unacknowledged_inputs.0.clear();
        return;
    }

    let mut frames = vec![];
    for input in inputs.buffer.iter() {
        frames.push(input.clone());
        unacknowledged_inputs.0.push(input.clone());
    }
    client.send_game_message(ClientToServerMessage::PlayerInputs(frames));
    inputs.buffer.clear();
}
