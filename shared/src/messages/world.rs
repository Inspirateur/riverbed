use std::collections::HashMap;

use bevy::ecs::message::Message;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::items::Stack;
use crate::world::chunk::Chunk;
use crate::world::pos::pos3d::{BlockPos, ChunkPos};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ServerToClientWorldUpdate {
    pub tick: u64,
    pub time: u64,
    pub new_map: HashMap<ChunkPos, Chunk>,
    pub item_stacks: Vec<ServerToClientItemStackUpdate>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Message)]
pub struct ServerToClientItemStackUpdate {
    pub id: u128,
    pub data: Option<(Stack, Vec3)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientToServerBlockInteraction {
    pub position: BlockPos,
    pub new_block: Block,
}
