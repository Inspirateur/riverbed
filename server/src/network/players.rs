use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use shared::messages::{ClientPlayerInput, PlayerId, ServerPlayerUpdate, ServerToClientMessage};
use shared::physics::{player_step::apply_player_input_step, MovementMode, PhysicsState};
use shared::world::realm::Realm;
use std::collections::HashMap;

use crate::world::voxel_world::VoxelWorld;

use super::dispatcher::NetworkPlayer;
use super::extensions::SendGameMessageExtension;

// Re-export from shared for backward compatibility
pub use shared::DEFAULT_SPAWN_POSITION;

/// Server-side physics state for a player entity
#[derive(Component, Debug, Clone)]
pub struct ServerPhysicsState {
    pub velocity: Vec3,
    pub movement_mode: MovementMode,
    pub on_ground: bool,
}

impl Default for ServerPhysicsState {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            movement_mode: MovementMode::Walking,
            on_ground: false,
        }
    }
}

/// The client's self-reported predicted position.
///
/// This is used for chunk streaming to ensure the client receives chunks
/// for where it *thinks* it is (after client-side prediction), not just
/// where the server's authoritative simulation says it is. This prevents
/// gaps in terrain when the client moves faster than the server can process.
#[derive(Component, Debug, Clone, Default)]
pub struct ClientPredictedPosition(pub Vec3);

#[derive(Debug, Clone)]
pub struct ServerPlayer {
    pub id: PlayerId,
    pub name: String,
    pub last_input_processed: u64,
    pub is_authenticated: bool,
}

impl ServerPlayer {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            name: format!("Player-{}", id),
            last_input_processed: 0,
            is_authenticated: false,
        }
    }
}

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

#[derive(Message, Debug)]
pub struct PlayerInputsEvent {
    pub client_id: ClientId,
    pub input: ClientPlayerInput,
}

/// Server-authoritative player input handling system.
///
/// This system receives player inputs from clients and simulates physics
/// authoritatively on the server. The server is the single source of truth
/// for player positions.
pub fn handle_player_inputs_system(
    mut events: MessageReader<PlayerInputsEvent>,
    mut registry: ResMut<PlayerRegistry>,
    mut player_query: Query<(
        &NetworkPlayer,
        &mut Transform,
        &mut ServerPhysicsState,
        &mut ClientPredictedPosition,
        &Realm,
    )>,
    world: Res<VoxelWorld>,
) {
    for ev in events.read() {
        let Some(player) = registry.get_player_mut(ev.client_id) else {
            warn!("Received input from unknown player {}", ev.client_id);
            continue;
        };

        if !player.is_authenticated {
            debug!(
                "Ignoring input from unauthenticated player {}",
                ev.client_id
            );
            continue;
        }

        let Some((_, mut transform, mut physics_state, mut predicted_pos, realm)) = player_query
            .iter_mut()
            .find(|(np, _, _, _, _)| np.client_id == ev.client_id)
        else {
            warn!(
                "No ECS entity found for authenticated player {}",
                ev.client_id
            );
            continue;
        };

        // Always update the client's predicted position for chunk streaming,
        // even if this input is stale. This ensures we stream chunks to where
        // the client thinks it is.
        predicted_pos.0 = ev.input.position;

        // Drop stale/duplicate inputs based on last processed timestamp.
        if ev.input.time_ms <= player.last_input_processed {
            continue;
        }

        let delta_seconds = ev.input.delta_ms as f32 / 1000.0;
        let state = PhysicsState {
            position: transform.translation,
            velocity: physics_state.velocity,
            movement_mode: physics_state.movement_mode,
            realm: *realm,
            on_ground: physics_state.on_ground,
        };

        let step = apply_player_input_step(
            &*world,
            &state,
            &ev.input.inputs,
            &ev.input.camera,
            delta_seconds,
        );

        transform.translation = step.position;
        transform.rotation = ev.input.camera.rotation;
        physics_state.velocity = step.velocity;
        physics_state.on_ground = step.on_ground;
        physics_state.movement_mode = step.movement_mode;

        player.last_input_processed = ev.input.time_ms;
    }
}

pub fn broadcast_player_updates_system(
    registry: Res<PlayerRegistry>,
    mut server: ResMut<RenetServer>,
    player_query: Query<(&NetworkPlayer, &Transform, &ServerPhysicsState)>,
) {
    // Only broadcast authenticated players
    for player in registry.players.values().filter(|p| p.is_authenticated) {
        // Get position, orientation, and physics state from ECS
        let (position, orientation, velocity, movement_mode) = player_query
            .iter()
            .find(|(np, _, _)| np.client_id == player.id)
            .map(|(_, t, ps)| (t.translation, t.rotation, ps.velocity, ps.movement_mode))
            .unwrap_or((
                DEFAULT_SPAWN_POSITION,
                Quat::IDENTITY,
                Vec3::ZERO,
                MovementMode::Walking,
            ));

        let update = ServerPlayerUpdate {
            id: player.id,
            position,
            velocity,
            orientation,
            movement_mode,
            last_ack_time: player.last_input_processed,
            inventory: Box::new([]), // TODO: Implement inventory sync
        };

        server.broadcast_game_message(ServerToClientMessage::PlayerUpdate(update));
    }
}
