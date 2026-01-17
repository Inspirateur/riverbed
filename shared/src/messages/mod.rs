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
    AuthRegisterRequest(AuthRegisterRequest),
    Exit,
    PlayerInputs(Vec<ClientPlayerInput>),
    SaveWorldRequest,
    BlockInteraction(ClientBlockInteraction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    AuthRegisterResponse(AuthRegisterResponse),
    WorldUpdate(ServerWorldUpdate),
    PlayerSpawn(ServerPlayerSpawn),
    PlayerUpdate(ServerPlayerUpdate),
    LogEvents(Vec<LogEvent>),
}
