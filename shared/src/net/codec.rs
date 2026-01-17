use bevy::log::{info, warn};
use bevy_renet::renet::{ChannelConfig, ClientId, RenetClient, RenetServer};
use bincode::ErrorKind;
use serde::{de::DeserializeOwned, Serialize};

use crate::messages::{ClientToServerMessage, ServerToClientMessage};
use crate::{game_message_to_payload, payload_to_game_message, utils::format_bytes, ChannelResolvableExt};

const LARGE_MESSAGE_BYTES: u64 = 10 * 1024;

/// Encode a typed game message into a channel id and compressed payload.
pub fn encode_message<M>(message: M) -> (u8, Vec<u8>)
where
    M: Serialize + ChannelResolvableExt,
{
    let channel = message.get_channel_id();
    let payload = game_message_to_payload(message);
    (channel, payload)
}

/// Decode a payload received from the network into a typed game message.
/// Returns `None` when no payload is present.
pub fn decode_payload<M, P>(
    payload: Option<P>,
    log_errors: bool,
) -> Option<Result<M, Box<ErrorKind>>>
where
    M: DeserializeOwned,
    P: AsRef<[u8]>,
{
    let Some(payload) = payload else {
        return None;
    };

    let result = payload_to_game_message::<M>(payload.as_ref());
    if log_errors {
        if let Err(err) = result.as_ref() {
            warn!("Error deserializing message: {:?}", err);
        }
    }
    Some(result)
}

/// Iterate through channel configs (skipping an optional exclusion) and decode the first
/// available payload using the provided receive function.
pub fn receive_from_channels<M, F, P>(
    channels: &[ChannelConfig],
    excluded_channel: Option<u8>,
    mut receive_payload: F,
    log_errors: bool,
) -> Option<Result<M, Box<ErrorKind>>>
where
    M: DeserializeOwned,
    F: FnMut(u8) -> Option<P>,
    P: AsRef<[u8]>,
{
    for channel in channels.iter().map(|c| c.channel_id) {
        if Some(channel) == excluded_channel {
            continue;
        }

        if let Some(result) = decode_payload(receive_payload(channel), log_errors) {
            return Some(result);
        }
    }

    None
}

/// Send a client-to-server message using the correct channel.
pub fn client_send(client: &mut RenetClient, message: ClientToServerMessage) {
    let (channel, payload) = encode_message(message);
    client.send_message(channel, payload);
}

/// Receive a server-to-client message on a specific channel.
pub fn client_receive_by_channel(
    client: &mut RenetClient,
    channel: u8,
) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>> {
    decode_payload(client.receive_message(channel), true)
}

/// Receive a server-to-client message from any channel except the excluded one.
pub fn client_receive_except_channel(
    client: &mut RenetClient,
    channels: &[ChannelConfig],
    excluded_channel: u8,
) -> Option<Result<ServerToClientMessage, Box<ErrorKind>>> {
    receive_from_channels(channels, Some(excluded_channel), |ch| client.receive_message(ch), true)
}

/// Send a server-to-client message to a specific client, logging large payloads.
pub fn server_send(
    server: &mut RenetServer,
    client_id: ClientId,
    message: ServerToClientMessage,
) {
    let (channel, payload) = encode_message(message);
    log_if_large(&payload);
    server.send_message(client_id, channel, payload);
}

/// Broadcast a server-to-client message to all clients, logging large payloads.
pub fn server_broadcast(server: &mut RenetServer, message: ServerToClientMessage) {
    let (channel, payload) = encode_message(message);
    log_if_large(&payload);
    server.broadcast_message(channel, payload);
}

/// Receive a client-to-server message from a specific channel.
pub fn server_receive_by_channel(
    server: &mut RenetServer,
    client_id: ClientId,
    channel: u8,
) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>> {
    decode_payload(server.receive_message(client_id, channel), true)
}

/// Receive a client-to-server message from the provided channels in order.
pub fn server_receive_any(
    server: &mut RenetServer,
    client_id: ClientId,
    channels: &[ChannelConfig],
) -> Option<Result<ClientToServerMessage, Box<ErrorKind>>> {
    receive_from_channels(channels, None, |ch| server.receive_message(client_id, ch), true)
}

fn log_if_large(payload: &[u8]) {
    let size = payload.len() as u64;
    if size > LARGE_MESSAGE_BYTES {
        info!("Sending game message of size: {}", format_bytes(size));
    }
}
