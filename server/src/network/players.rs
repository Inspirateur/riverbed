//! Player management for the server.
//!
//! Tracks connected players, their positions, and handles player input processing.

use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use shared::messages::{
    PlayerId, PlayerFrameInput, PlayerUpdateEvent, ServerToClientMessage, TransmittableAction,
};
use std::collections::HashMap;

use super::extensions::SendGameMessageExtension;

/// Represents a player connected to the server.
#[derive(Debug, Clone)]
pub struct ServerPlayer {
    pub id: PlayerId,
    pub name: String,
    pub position: Vec3,
    pub orientation: Quat,
    pub is_flying: bool,
    /// The last input timestamp we've processed for this player
    pub last_input_processed: u64,
}

impl ServerPlayer {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            name: format!("Player-{}", id),
            position: Vec3::new(280., 500., -150.), // Default spawn position
            orientation: Quat::IDENTITY,
            is_flying: false,
            last_input_processed: 0,
        }
    }
}

/// Registry of all connected players.
#[derive(Resource, Default)]
pub struct PlayerRegistry {
    pub players: HashMap<PlayerId, ServerPlayer>,
}

impl PlayerRegistry {
    /// Add a new player to the registry
    pub fn add_player(&mut self, client_id: ClientId) {
        let player = ServerPlayer::new(client_id);
        info!("Adding player {} to registry", client_id);
        self.players.insert(client_id, player);
    }

    /// Remove a player from the registry
    pub fn remove_player(&mut self, client_id: ClientId) {
        info!("Removing player {} from registry", client_id);
        self.players.remove(&client_id);
    }

    /// Get a mutable reference to a player
    pub fn get_player_mut(&mut self, client_id: ClientId) -> Option<&mut ServerPlayer> {
        self.players.get_mut(&client_id)
    }

    /// Get an immutable reference to a player
    pub fn get_player(&self, client_id: ClientId) -> Option<&ServerPlayer> {
        self.players.get(&client_id)
    }

    /// Get player position by client ID (for chunk broadcasting)
    pub fn get_player_position(&self, client_id: ClientId) -> Option<Vec3> {
        self.players.get(&client_id).map(|p| p.position)
    }
}

/// Event fired when player inputs are received from a client.
#[derive(Message, Debug)]
pub struct PlayerInputsEvent {
    pub client_id: ClientId,
    pub input: PlayerFrameInput,
}

/// Movement constants
const WALK_SPEED: f32 = 7.0;
const FLY_SPEED: f32 = 15.0;

/// Process player inputs and update positions.
/// This is a simplified server-side simulation.
pub fn handle_player_inputs_system(
    mut events: MessageReader<PlayerInputsEvent>,
    mut registry: ResMut<PlayerRegistry>,
) {
    for ev in events.read() {
        let Some(player) = registry.get_player_mut(ev.client_id) else {
            warn!("Received input from unknown player {}", ev.client_id);
            continue;
        };

        // Calculate movement direction from inputs
        let mut move_dir = Vec3::ZERO;
        let delta_secs = ev.input.delta_ms as f32 / 1000.0;

        // Get camera forward/right vectors (horizontal only for walking)
        let forward = ev.input.camera.forward().as_vec3();
        let right = ev.input.camera.right().as_vec3();
        let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        for action in &ev.input.inputs {
            match action {
                TransmittableAction::MoveForward => move_dir += forward_flat,
                TransmittableAction::MoveBackward => move_dir -= forward_flat,
                TransmittableAction::MoveRight => move_dir += right_flat,
                TransmittableAction::MoveLeft => move_dir -= right_flat,
                TransmittableAction::JumpOrFlyUp => {
                    if player.is_flying {
                        move_dir += Vec3::Y;
                    }
                }
                TransmittableAction::CrouchOrFlyDown => {
                    if player.is_flying {
                        move_dir -= Vec3::Y;
                    }
                }
                TransmittableAction::ToggleFlyMode => {
                    player.is_flying = !player.is_flying;
                }
                TransmittableAction::Hit | TransmittableAction::Modify => {
                    // Block interactions - handled separately
                }
            }
        }

        // Normalize and apply speed
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
            let speed = if player.is_flying { FLY_SPEED } else { WALK_SPEED };
            player.position += move_dir * speed * delta_secs;
        }

        // Update orientation from camera
        player.orientation = ev.input.camera.rotation;
        player.last_input_processed = ev.input.time_ms;
    }
}

/// Broadcast player updates to all connected clients.
pub fn broadcast_player_updates_system(
    registry: Res<PlayerRegistry>,
    mut server: ResMut<RenetServer>,
) {
    // For each player, broadcast their current state to all clients
    for player in registry.players.values() {
        let update = PlayerUpdateEvent {
            id: player.id,
            position: player.position,
            orientation: player.orientation,
            last_ack_time: player.last_input_processed,
            inventory: Box::new([]), // TODO: Implement inventory sync
        };

        server.broadcast_game_message(ServerToClientMessage::PlayerUpdate(update));
    }
}
