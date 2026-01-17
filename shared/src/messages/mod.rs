mod auth;
mod chat;
pub mod player;
mod world;

pub use auth::*;
pub use chat::*;
pub use player::*;
use serde::{Deserialize, Serialize};
pub use world::*;

pub type PlayerId = u64;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientToServerMessage {
    AuthRegisterRequest(AuthRegisterRequest),
    ChatMessage(ChatMessageRequest),
    Exit,
    PlayerInputs(Vec<ClientPlayerInput>),
    SaveWorldRequest,
    BlockInteraction(ClientBlockInteraction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    AuthRegisterResponse(AuthRegisterResponse),
    ChatHistory(ServerChatHistory),
    WorldUpdate(ServerWorldUpdate),
    PlayerSpawn(ServerPlayerSpawn),
    PlayerUpdate(ServerPlayerUpdate),
}
