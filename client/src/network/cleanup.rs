use crate::network::{SendGameMessageExtension, TargetServer, TargetServerState};
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::ClientToServerMessage;

use super::{buffered_client::PlayerTickInputsBuffer, UnacknowledgedInputs};

#[derive(Message, Debug)]
pub struct ExitRequestEvent;

pub fn handle_exit_request(
    mut ev_exit: MessageReader<ExitRequestEvent>,
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    mut unacknowledged_inputs: ResMut<UnacknowledgedInputs>,
    mut current_frame: ResMut<PlayerTickInputsBuffer>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for _ in ev_exit.read() {
        terminate_server_connection(
            &mut client,
            &mut target,
            &mut unacknowledged_inputs,
            &mut current_frame,
        );
        app_exit.write(AppExit::Success);
    }
}

pub fn on_app_exit(
    mut ev_app_exit: MessageReader<AppExit>,
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    mut unacknowledged_inputs: ResMut<UnacknowledgedInputs>,
    mut current_frame: ResMut<PlayerTickInputsBuffer>,
) {
    for _ in ev_app_exit.read() {
        if target.session_token.is_some() {
            terminate_server_connection(
                &mut client,
                &mut target,
                &mut unacknowledged_inputs,
                &mut current_frame,
            );
        }
    }
}

fn terminate_server_connection(
    client: &mut ResMut<RenetClient>,
    target: &mut ResMut<TargetServer>,
    unacknowledged_inputs: &mut ResMut<UnacknowledgedInputs>,
    current_frame: &mut ResMut<PlayerTickInputsBuffer>,
) {
    info!("Terminating server connection");
    
    if !client.is_disconnected() {
        client.send_game_message(ClientToServerMessage::Exit);
    }

    target.address = None;
    target.username = None;
    target.session_token = None;
    target.state = TargetServerState::Initial;

    unacknowledged_inputs.0.clear();
    current_frame.buffer.clear();
}
