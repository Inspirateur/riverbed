use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use serde::{Deserialize, Serialize};

use crate::items::{Stack, item_slots::inventory_serde};
use crate::physics::MovementMode;

use super::PlayerId;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub enum TransmittableAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    JumpOrFlyUp,
    CrouchOrFlyDown,
    ToggleFlyMode,
    Hit,
    Modify,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct PlayerSave {
    pub position: Vec3,
    pub camera_transform: Transform,
    pub is_flying: bool,
}

#[derive(Message, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ServerPlayerSpawn {
    pub id: PlayerId,
    pub name: String,
    pub data: PlayerSave,
}

#[derive(Message, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ServerPlayerUpdate {
    pub id: PlayerId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub movement_mode: MovementMode,
    pub last_ack_time: u64,
    #[serde(with = "inventory_serde")]
    pub inventory: Box<[Stack]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ClientPlayerInput {
    pub time_ms: u64,
    pub delta_ms: u64,
    pub inputs: HashSet<TransmittableAction>,
    pub camera: Transform,
    pub hotbar_slot: u32,
    /// Client's predicted position at time of input (for validation)
    pub position: Vec3,
    /// Client's current velocity (for physics simulation)
    pub velocity: Vec3,
}
