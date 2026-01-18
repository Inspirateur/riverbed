use serde::{Deserialize, Serialize};

use super::{ClientToServerMessage, ServerToClientMessage, ServerToClientPlayerSpawn};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ClientToServerAuthRequest {
    pub username: String,
}

impl From<ClientToServerAuthRequest> for ClientToServerMessage {
    fn from(val: ClientToServerAuthRequest) -> Self {
        ClientToServerMessage::AuthRequest(val)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ServerToClientAuthResponse {
    pub username: String,
    pub session_token: u64,
    pub tick: u64,
    pub timestamp_ms: u64,
    pub players: Vec<ServerToClientPlayerSpawn>,
    pub world_seed: u32,
}

impl From<ServerToClientAuthResponse> for ServerToClientMessage {
    fn from(val: ServerToClientAuthResponse) -> Self {
        ServerToClientMessage::AuthResponse(val)
    }
}
