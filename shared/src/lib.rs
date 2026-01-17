pub mod asset_processing;
pub mod block;
pub mod items;
pub mod logging;
pub mod messages;
pub mod net;
pub mod physics;
pub mod world;
pub mod utils;

use std::time::Duration;

use bevy::log::debug;
use bincode::Options;
use block::{Block, BlockFamily};
use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ConnectionConfig, SendType};
use crate::utils::format_bytes;

use crate::messages::{ClientToServerMessage, ServerToClientMessage};

// Network protocol constants
pub const PROTOCOL_ID: u64 = 0;
pub const TICKS_PER_SECOND: u64 = 20;
pub const RENDER_DISTANCE: i32 = 32;

// Player movement constants
pub const WALK_SPEED: f32 = 7.0;
pub const FLY_SPEED: f32 = 500.0;
pub const FLY_VERTICAL_SPEED: f32 = 100.0;

// Player spawn position (shared between client and server)
pub const DEFAULT_SPAWN_POSITION: Vec3 = Vec3::new(280., 500., -150.);

// Re-export physics constants for convenience
pub use physics::{PLAYER_GRAVITY, PLAYER_JUMP_FORCE, PLAYER_AABB, ACC_MULT};

// Error message constants
pub const UNIX_EPOCH_TIME_ERROR: &str = "System time is before UNIX_EPOCH";
pub const SOCKET_LOCAL_ADDR_ERROR: &str = "Failed to retrieve local address for UDP socket";
pub const SOCKET_BIND_ERROR: &str = "Failed to bind UDP socket";
pub const TARGET_SERVER_ADDR_ERROR: &str =
    "Target server address missing when initializing connection";
pub const NETCODE_CLIENT_TRANSPORT_ERROR: &str = "Failed to create Netcode client transport";
pub const NETCODE_SERVER_TRANSPORT_ERROR: &str = "Failed to create Netcode server transport";
pub const USERNAME_MISSING_AUTHENTICATED_ERROR: &str =
    "Username missing while handling authenticated session token";

#[derive(Resource)]
pub struct GameServerConfig {
    pub world_name: String,
    pub is_solo: bool,
    pub broadcast_render_distance: i32,
}

const MAX_MEMORY: usize = 128 * 1024 * 1024;
const RESEND_TIME: Duration = Duration::from_millis(300);
const AVAILABLE_BYTES_PER_TICK: u64 = 5 * 1024 * 1024;

pub const CTS_STANDARD_CHANNEL: u8 = 0;
pub const CTS_AUTH_CHANNEL: u8 = 1;

pub fn get_customized_client_to_server_channels() -> Vec<ChannelConfig> {
    vec![
        ChannelConfig {
            channel_id: CTS_STANDARD_CHANNEL,
            max_memory_usage_bytes: MAX_MEMORY,
            send_type: SendType::ReliableOrdered {
                resend_time: RESEND_TIME,
            },
        },
        ChannelConfig {
            channel_id: CTS_AUTH_CHANNEL,
            max_memory_usage_bytes: MAX_MEMORY,
            send_type: SendType::ReliableOrdered {
                resend_time: RESEND_TIME,
            },
        },
    ]
}

pub const STC_STANDARD_CHANNEL: u8 = 0;
pub const STC_CHUNK_DATA_CHANNEL: u8 = 1;
pub const STC_AUTH_CHANNEL: u8 = 2;
pub const STC_LOG_EVENTS_CHANNEL: u8 = 3;

pub fn get_customized_server_to_client_channels() -> Vec<ChannelConfig> {
    vec![
        ChannelConfig {
            channel_id: STC_STANDARD_CHANNEL,
            max_memory_usage_bytes: MAX_MEMORY,
            send_type: SendType::ReliableOrdered {
                resend_time: RESEND_TIME,
            },
        },
        ChannelConfig {
            channel_id: STC_CHUNK_DATA_CHANNEL,
            max_memory_usage_bytes: MAX_MEMORY,
            send_type: SendType::ReliableOrdered {
                resend_time: RESEND_TIME,
            },
        },
        ChannelConfig {
            channel_id: STC_AUTH_CHANNEL,
            max_memory_usage_bytes: MAX_MEMORY,
            send_type: SendType::ReliableOrdered {
                resend_time: RESEND_TIME,
            },
        },
        ChannelConfig {
            channel_id: STC_LOG_EVENTS_CHANNEL,
            max_memory_usage_bytes: MAX_MEMORY,
            send_type: SendType::ReliableOrdered {
                resend_time: RESEND_TIME,
            },
        },
    ]
}

pub fn get_shared_renet_config() -> ConnectionConfig {
    ConnectionConfig {
        client_channels_config: get_customized_client_to_server_channels(),
        server_channels_config: get_customized_server_to_client_channels(),
        available_bytes_per_tick: AVAILABLE_BYTES_PER_TICK,
    }
}

pub fn game_message_to_payload<T: serde::Serialize>(message: T) -> Vec<u8> {
    let payload = bincode::options().serialize(&message).unwrap();
    let output = lz4::block::compress(&payload, None, true).unwrap();
    if payload.len() > 1024 {
        debug!(
            "Original payload size: {}",
            format_bytes(payload.len() as u64)
        );
        debug!(
            "Compressed payload of size: {}",
            format_bytes(output.len() as u64)
        );
    }
    output
}

pub fn payload_to_game_message<T: serde::de::DeserializeOwned>(
    payload: &[u8],
) -> Result<T, bincode::Error> {
    let decompressed_payload = lz4::block::decompress(payload, None)?;
    bincode::options().deserialize(&decompressed_payload)
}

pub trait ChannelResolvableExt {
    fn get_channel_id(&self) -> u8;
}

impl ChannelResolvableExt for ClientToServerMessage {
    fn get_channel_id(&self) -> u8 {
        match self {
            ClientToServerMessage::AuthRegisterRequest(_) => CTS_AUTH_CHANNEL,
            _ => CTS_STANDARD_CHANNEL,
        }
    }
}

impl ChannelResolvableExt for ServerToClientMessage {
    fn get_channel_id(&self) -> u8 {
        match self {
            ServerToClientMessage::WorldUpdate(_) => STC_CHUNK_DATA_CHANNEL,
            ServerToClientMessage::AuthRegisterResponse(_) => STC_AUTH_CHANNEL,
            ServerToClientMessage::LogEvents(_) => STC_LOG_EVENTS_CHANNEL,
            _ => STC_STANDARD_CHANNEL,
        }
    }
}
