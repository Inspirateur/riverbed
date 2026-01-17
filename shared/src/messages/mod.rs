mod auth;
mod chat;
pub mod player;
mod world;

pub use auth::*;
pub use chat::*;
pub use player::*;
use serde::{Deserialize, Serialize};
pub use world::*;

/// Unique identifier for a player in the game.
/// 
/// This is the same type as `bevy_renet::renet::ClientId` (both are `u64`).
/// We use a separate alias here for semantic clarity in game logic vs networking code,
/// but they can be used interchangeably.
pub type PlayerId = u64;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientToServerMessage {
    AuthRegisterRequest(AuthRegisterRequest),
    ChatMessage(ChatMessageRequest),
    Exit,
    PlayerInputs(Vec<PlayerFrameInput>),
    SaveWorldRequest,
    BlockInteraction(BlockInteraction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    AuthRegisterResponse(AuthRegisterResponse),
    ChatConversation(ChatConversation),
    WorldUpdate(WorldUpdate),
    PlayerSpawn(PlayerSpawnEvent),
    PlayerUpdate(PlayerUpdateEvent),
}
