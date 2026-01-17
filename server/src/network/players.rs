use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use shared::messages::{
    PlayerId, ClientPlayerInput, ServerPlayerUpdate, ServerToClientMessage, TransmittableAction,
};
use shared::physics::{
    actions_to_movement_input, MovementMode, PhysicsState, simulate_physics_step,
};
use shared::world::realm::Realm;
use std::collections::HashMap;

use crate::world::voxel_world::VoxelWorld;

use super::dispatcher::NetworkPlayer;
use super::extensions::SendGameMessageExtension;

pub const DEFAULT_SPAWN_POSITION: Vec3 = Vec3::new(280., 500., -150.);

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
    mut player_query: Query<(&NetworkPlayer, &mut Transform, &mut ServerPhysicsState, &Realm)>,
    world: Res<VoxelWorld>,
) {
    for ev in events.read() {
        let Some(player) = registry.get_player_mut(ev.client_id) else {
            warn!("Received input from unknown player {}", ev.client_id);
            continue;
        };

        if !player.is_authenticated {
            debug!("Ignoring input from unauthenticated player {}", ev.client_id);
            continue;
        }

        let Some((_, mut transform, mut physics_state, realm)) = player_query
            .iter_mut()
            .find(|(np, _, _, _)| np.client_id == ev.client_id)
        else {
            warn!("No ECS entity found for authenticated player {}", ev.client_id);
            continue;
        };

        // Handle fly mode toggle
        if ev.input.inputs.contains(&TransmittableAction::ToggleFlyMode) {
            physics_state.movement_mode = match physics_state.movement_mode {
                MovementMode::Walking => MovementMode::Flying,
                MovementMode::Flying => MovementMode::Walking,
            };
            // Reset velocity when toggling fly mode
            if physics_state.movement_mode == MovementMode::Walking {
                physics_state.velocity = Vec3::ZERO;
            }
        }

        let delta_seconds = ev.input.delta_ms as f32 / 1000.0;
        
        // Convert inputs to movement input
        let movement_input = actions_to_movement_input(&ev.input.inputs, &ev.input.camera);
        
        // Build physics state for simulation
        let state = PhysicsState {
            position: transform.translation,
            velocity: physics_state.velocity,
            movement_mode: physics_state.movement_mode,
            realm: *realm,
            on_ground: physics_state.on_ground,
        };
        
        // Run authoritative physics simulation
        let result = simulate_physics_step(&*world, &state, &movement_input, delta_seconds);
        
        // Apply results
        transform.translation = result.new_position;
        transform.rotation = ev.input.camera.rotation;
        physics_state.velocity = result.new_velocity;
        physics_state.on_ground = result.on_ground;
        
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
            .unwrap_or((DEFAULT_SPAWN_POSITION, Quat::IDENTITY, Vec3::ZERO, MovementMode::Walking));

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
