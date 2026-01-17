use crate::network::{SendGameMessageExtension, TargetServer, TargetServerState};
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::ClientToServerMessage;

use super::{buffered_client::PlayerTickInputsBuffer, UnacknowledgedInputs};

/// Event that triggers graceful disconnection from the server.
/// Send this event when the player wants to quit or return to menu.
#[derive(Message, Debug)]
pub struct ExitRequestEvent;

/// System that handles ExitRequestEvent by terminating the server connection.
pub fn handle_exit_request(
    mut ev_exit: MessageReader<ExitRequestEvent>,
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    mut unacknowledged_inputs: ResMut<UnacknowledgedInputs>,
    mut current_frame: ResMut<PlayerTickInputsBuffer>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for _ in ev_exit.read() {
        terminate_server_connection_inner(
            &mut client,
            &mut target,
            &mut unacknowledged_inputs,
            &mut current_frame,
        );
        // Exit the application after cleanup
        app_exit.write(AppExit::Success);
    }
}

/// System that runs on app exit to ensure we send Exit message to server.
/// This handles cases like window close button, Alt+F4, etc.
pub fn on_app_exit(
    mut ev_app_exit: MessageReader<AppExit>,
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    mut unacknowledged_inputs: ResMut<UnacknowledgedInputs>,
    mut current_frame: ResMut<PlayerTickInputsBuffer>,
) {
    for _ in ev_app_exit.read() {
        // Only send if we're connected
        if target.session_token.is_some() {
            terminate_server_connection_inner(
                &mut client,
                &mut target,
                &mut unacknowledged_inputs,
                &mut current_frame,
            );
        }
    }
}

/// Inner function that performs the actual connection termination.
fn terminate_server_connection_inner(
    client: &mut ResMut<RenetClient>,
    target: &mut ResMut<TargetServer>,
    unacknowledged_inputs: &mut ResMut<UnacknowledgedInputs>,
    current_frame: &mut ResMut<PlayerTickInputsBuffer>,
) {
    info!("Terminating server connection");
    
    // Send exit message to server if connected
    if !client.is_disconnected() {
        client.send_game_message(ClientToServerMessage::Exit);
    }

    // Reset connection state
    target.address = None;
    target.username = None;
    target.session_token = None;
    target.state = TargetServerState::Initial;

    // Clear input buffers
    unacknowledged_inputs.0.clear();
    current_frame.buffer.clear();
}
