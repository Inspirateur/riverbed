use crate::network::{SendGameMessageExtension, TargetServer, TargetServerState};
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::ClientToServerMessage;
use shared::net::input_history::InputHistory;

#[derive(Message, Debug)]
pub struct ExitRequestEvent;

pub fn handle_exit_request(
    mut ev_exit: MessageReader<ExitRequestEvent>,
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    mut input_history: ResMut<InputHistory>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for _ in ev_exit.read() {
        terminate_server_connection(
            &mut client,
            &mut target,
            &mut input_history,
        );
        app_exit.write(AppExit::Success);
    }
}

pub fn on_app_exit(
    mut ev_app_exit: MessageReader<AppExit>,
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    mut input_history: ResMut<InputHistory>,
) {
    for _ in ev_app_exit.read() {
        if target.session_token.is_some() {
            terminate_server_connection(
                &mut client,
                &mut target,
                &mut input_history,
            );
        }
    }
}

fn terminate_server_connection(
    client: &mut ResMut<RenetClient>,
    target: &mut ResMut<TargetServer>,
    input_history: &mut ResMut<InputHistory>,
) {
    info!("Terminating server connection");
    
    if !client.is_disconnected() {
        client.send_game_message(ClientToServerMessage::Exit);
    }

    target.address = None;
    target.username = None;
    target.session_token = None;
    target.state = TargetServerState::Initial;

    input_history.clear_all();
}
