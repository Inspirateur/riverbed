use bevy_renet::renet::RenetClient;
use bincode::ErrorKind;
use shared::{
    get_customized_server_to_client_channels,
    messages::{ClientToServerMessage, ServerToClientMessage},
    net::codec,
};

pub trait SendGameMessageExtension {
    fn send_game_message(&mut self, message: ClientToServerMessage);
    fn receive_game_message_by_channel(
        &mut self,
        channel: u8,
    ) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>>;
    fn receive_game_message_except_channel(
        &mut self,
        channel: u8,
    ) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>>;
    // fn receive_game_message(&mut self) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>>;
}

impl SendGameMessageExtension for RenetClient {
    fn send_game_message(&mut self, message: ClientToServerMessage) {
        codec::client_send(self, message);
    }

    fn receive_game_message_by_channel(
        &mut self,
        channel: u8,
    ) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>> {
        codec::client_receive_by_channel(self, channel)
    }

    fn receive_game_message_except_channel(
        &mut self,
        excluded_channel_id: u8,
    ) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>> {
        let channels = get_customized_server_to_client_channels();
        codec::client_receive_except_channel(self, &channels, excluded_channel_id)
    }
}
