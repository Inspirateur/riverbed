use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::{ClientToServerMessage, PlayerFrameInput};

use crate::agents::key_binds::KeyBinds;
use crate::agents::PlayerControlled;
use crate::render::FpsCam;
use crate::ui::SelectedHotbarSlot;

use super::buffered_client::{CurrentFrameInputs, CurrentFrameInputsExt, PlayerTickInputsBuffer, SyncTime, SyncTimeExt};
use super::SendGameMessageExtension;

// vector of time_ms values for inputs that have not been acknowledged by the server
#[derive(Debug, Default, Resource)]
pub struct UnacknowledgedInputs(pub Vec<PlayerFrameInput>);

/// System that runs at the start of each frame to prepare input collection.
/// Advances the sync time and buffers the previous frame's inputs.
pub fn pre_input_update_system(
    mut frame_inputs: ResMut<CurrentFrameInputs>,
    mut tick_buffer: ResMut<PlayerTickInputsBuffer>,
    mut sync_time: ResMut<SyncTime>,
) {
    sync_time.advance();

    // Buffer the completed frame's inputs
    let inputs_of_last_frame = frame_inputs.0.clone();
    tick_buffer.buffer.push(inputs_of_last_frame);
    
    // Reset for new frame
    frame_inputs.reset(sync_time.curr_time_ms, sync_time.delta());
}

/// System that captures keyboard and mouse inputs and converts them to TransmittableActions.
/// Only runs when the cursor is grabbed (player is in game, not in menu).
pub fn capture_player_inputs_system(
    key_binds: Res<KeyBinds>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut frame_inputs: ResMut<CurrentFrameInputs>,
) {
    // Skip if delta is 0 (frame not properly initialized)
    if frame_inputs.0.delta_ms == 0 {
        return;
    }

    // Capture keyboard inputs
    for keycode in keyboard.get_pressed() {
        if let Some(action) = key_binds.keycode_to_action(*keycode) {
            frame_inputs.0.inputs.insert(action);
        }
    }

    // Capture mouse button inputs
    for button in mouse.get_pressed() {
        if let Some(action) = key_binds.mousebutton_to_action(*button) {
            frame_inputs.0.inputs.insert(action);
        }
    }

    // Also capture just-pressed for toggle actions (like ToggleFlyMode)
    for keycode in keyboard.get_just_pressed() {
        if let Some(action) = key_binds.keycode_to_action(*keycode) {
            frame_inputs.0.inputs.insert(action);
        }
    }
}

/// System that updates frame inputs with camera transform and hotbar selection.
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
    // for frame in frames.iter() {
    //     debug!(
    //         "Sending input: {:?} | {:?} | {:?}",
    //         frame.time_ms, frame.inputs, frame.position
    //     );
    // }
    client.send_game_message(ClientToServerMessage::PlayerInputs(frames));
    inputs.buffer.clear();
}
