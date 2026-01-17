use bincode::ErrorKind;
use bevy_renet::renet::{ClientId, RenetServer};
use shared::{
    get_customized_client_to_server_channels,
    messages::{ClientToServerMessage, ServerToClientMessage},
    net::codec,
};

pub trait SendGameMessageExtension {
    fn send_game_message(&mut self, client_id: ClientId, message: ServerToClientMessage);
    fn broadcast_game_message(&mut self, message: ServerToClientMessage);
    fn receive_game_message(
        &mut self,
        client_id: ClientId,
    ) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>>;
    fn receive_game_message_by_channel(
        &mut self,
        client_id: ClientId,
        channel: u8,
    ) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>>;
    fn receive_game_message_except_channel(
        &mut self,
        client_id: ClientId,
        excluded_channel_id: u8,
    ) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>>;
}

impl SendGameMessageExtension for RenetServer {
    fn send_game_message(&mut self, client_id: ClientId, message: ServerToClientMessage) {
        codec::server_send(self, client_id, message);
    }

    fn broadcast_game_message(&mut self, message: ServerToClientMessage) {
        codec::server_broadcast(self, message);
    }

    fn receive_game_message_by_channel(
        &mut self,
        client_id: ClientId,
        channel: u8,
    ) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>> {
        codec::server_receive_by_channel(self, client_id, channel)
    }

    fn receive_game_message(
        &mut self,
        client_id: ClientId,
    ) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>> {
        let channels = get_customized_client_to_server_channels();
        codec::server_receive_any(self, client_id, &channels)
    }

    fn receive_game_message_except_channel(
        &mut self,
        client_id: ClientId,
        excluded_channel_id: u8,
    ) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>> {
        let channels = get_customized_client_to_server_channels();
        for channel in channels.iter().filter(|c| c.channel_id != excluded_channel_id) {
            if let Some(res) = codec::server_receive_by_channel(self, client_id, channel.channel_id) {
                return Some(res);
            }
        }
        None
    }
}
