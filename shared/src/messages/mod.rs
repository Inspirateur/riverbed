mod auth;
pub mod player;
mod world;

pub use auth::*;
pub use player::*;
use serde::{Deserialize, Serialize};
pub use world::*;

use crate::logging::logging::LogEvent;

pub type PlayerId = u64;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientToServerMessage {
    AuthRequest(ClientToServerAuthRequest),
    Exit,
    PlayerInputs(Vec<ClientToServerPlayerInput>),
    SaveWorldRequest,
    BlockInteraction(ClientToServerBlockInteraction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    AuthResponse(ServerToClientAuthResponse),
    WorldUpdate(ServerToClientWorldUpdate),
    PlayerSpawn(ServerToClientPlayerSpawn),
    PlayerUpdate(ServerToClientPlayerUpdate),
    LogEvents(Vec<LogEvent>),
}
