//! Player management for the server.
//!
//! Tracks connected players and handles player input processing.
//! 
//! # Architecture
//! 
//! Player state is split between two locations:
//! - **ECS entities**: Position (`Transform`) and realm (`Realm`) live on entities with
//!   the `NetworkPlayer` component. This is the single source of truth for spatial data.
//! - **PlayerRegistry**: Metadata like name, authentication status, and input timing.
//!   This provides O(1) lookup by `ClientId` without needing ECS queries.
//!
//! When a player authenticates, we spawn an ECS entity and add them to the registry.
//! When they disconnect, both are cleaned up.

use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use shared::messages::{
    PlayerId, PlayerFrameInput, PlayerUpdateEvent, ServerToClientMessage, TransmittableAction,
};
use std::collections::HashMap;

use super::dispatcher::NetworkPlayer;
use super::extensions::SendGameMessageExtension;

/// Default spawn position for new players.
/// TODO: This should be loaded from world data or calculated based on terrain.
pub const DEFAULT_SPAWN_POSITION: Vec3 = Vec3::new(280., 500., -150.);

/// Metadata for a player connected to the server.
/// 
/// Note: Position and orientation are stored on the player's ECS entity (`Transform`),
/// not here. This struct only contains non-spatial metadata.
#[derive(Debug, Clone)]
pub struct ServerPlayer {
    /// The player's network client ID. This is the same as `PlayerId` (both are `u64`).
    pub id: PlayerId,
    /// Display name chosen during authentication.
    pub name: String,
    /// Whether the player is currently flying (affects movement physics).
    pub is_flying: bool,
    /// The timestamp (ms) of the last input we've processed for this player.
    /// Used for acknowledgment in `PlayerUpdateEvent`.
    pub last_input_processed: u64,
    /// Whether the player has completed the authentication handshake.
    pub is_authenticated: bool,
}

impl ServerPlayer {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            name: format!("Player-{}", id),
            is_flying: false,
            last_input_processed: 0,
            is_authenticated: false,
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

    /// Check if a player is authenticated
    pub fn is_authenticated(&self, client_id: ClientId) -> bool {
        self.players
            .get(&client_id)
            .map(|p| p.is_authenticated)
            .unwrap_or(false)
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
/// 
/// This system reads input events, updates the player's ECS `Transform` for position,
/// and updates `PlayerRegistry` for metadata (flying state, last processed input).
pub fn handle_player_inputs_system(
    mut events: MessageReader<PlayerInputsEvent>,
    mut registry: ResMut<PlayerRegistry>,
    mut player_transforms: Query<(&NetworkPlayer, &mut Transform)>,
) {
    for ev in events.read() {
        let Some(player) = registry.get_player_mut(ev.client_id) else {
            warn!("Received input from unknown player {}", ev.client_id);
            continue;
        };

        // Skip unauthenticated players
        if !player.is_authenticated {
            debug!("Ignoring input from unauthenticated player {}", ev.client_id);
            continue;
        }

        // Find the player's ECS entity to update Transform
        let Some((_, mut transform)) = player_transforms
            .iter_mut()
            .find(|(np, _)| np.client_id == ev.client_id)
        else {
            warn!("No ECS entity found for authenticated player {}", ev.client_id);
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

        // Normalize and apply speed to ECS Transform (single source of truth for position)
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
            let speed = if player.is_flying { FLY_SPEED } else { WALK_SPEED };
            transform.translation += move_dir * speed * delta_secs;
        }

        // Update orientation on ECS Transform
        transform.rotation = ev.input.camera.rotation;
        
        // Update metadata in registry
        player.last_input_processed = ev.input.time_ms;
    }
}

/// Broadcast player updates to all connected clients.
/// 
/// Reads position/orientation from ECS `Transform` and metadata from `PlayerRegistry`.
pub fn broadcast_player_updates_system(
    registry: Res<PlayerRegistry>,
    mut server: ResMut<RenetServer>,
    player_transforms: Query<(&NetworkPlayer, &Transform)>,
) {
    // Only broadcast authenticated players
    for player in registry.players.values().filter(|p| p.is_authenticated) {
        // Get position and orientation from ECS Transform
        let (position, orientation) = player_transforms
            .iter()
            .find(|(np, _)| np.client_id == player.id)
            .map(|(_, t)| (t.translation, t.rotation))
            .unwrap_or((DEFAULT_SPAWN_POSITION, Quat::IDENTITY));

        let update = PlayerUpdateEvent {
            id: player.id,
            position,
            orientation,
            last_ack_time: player.last_input_processed,
            inventory: Box::new([]), // TODO: Implement inventory sync
        };

        server.broadcast_game_message(ServerToClientMessage::PlayerUpdate(update));
    }
}
