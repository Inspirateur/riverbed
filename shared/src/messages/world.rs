use std::collections::HashMap;

use bevy::math::{IVec3, Vec3};
use bevy::ecs::message::Message;
use serde::{Deserialize, Serialize};

use crate::world::pos::ChunkPos;

/// WorldUpdate is a message sent from the server to the client to update the client's world state.
/// Only chunks which have been updated since the last message are sent.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct WorldUpdate {
    pub tick: u64,
    pub time: u64,
    pub new_map: HashMap<ChunkPos, ServerChunk>,
    pub item_stacks: Vec<ItemStackUpdateEvent>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Message)]
pub struct ItemStackUpdateEvent {
    pub id: u128,
    /// `None` if the stack has been deleted, `Some` if it has been updated in any way (position, number of items...)
    pub data: Option<(ItemStack, Vec3)>,
}

pub struct ChunkUpdate {
    pub position: IVec3,
    pub chunk: ServerChunk,
}
