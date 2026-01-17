use serde::{Deserialize, Serialize};

use super::{ClientToServerMessage, ServerPlayerSpawn, ServerToClientMessage};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AuthRegisterRequest {
    pub username: String,
}

impl From<AuthRegisterRequest> for ClientToServerMessage {
    fn from(val: AuthRegisterRequest) -> Self {
        ClientToServerMessage::AuthRegisterRequest(val)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AuthRegisterResponse {
    pub username: String,
    pub session_token: u64,
    pub tick: u64,
    pub timestamp_ms: u64,
    pub players: Vec<ServerPlayerSpawn>,
    pub world_seed: u32,
}

impl From<AuthRegisterResponse> for ServerToClientMessage {
    fn from(val: AuthRegisterResponse) -> Self {
        ServerToClientMessage::AuthRegisterResponse(val)
    }
}
