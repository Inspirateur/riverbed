use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use shared::messages::{
    PlayerId, ClientPlayerInput, ServerPlayerUpdate, ServerToClientMessage, TransmittableAction,
};
use shared::{WALK_SPEED, FLY_SPEED};
use std::collections::HashMap;

use super::dispatcher::NetworkPlayer;
use super::extensions::SendGameMessageExtension;

pub const DEFAULT_SPAWN_POSITION: Vec3 = Vec3::new(280., 500., -150.);

#[derive(Debug, Clone)]
pub struct ServerPlayer {
    pub id: PlayerId,
    pub name: String,
    pub is_flying: bool,
    pub last_input_processed: u64,
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

        if !player.is_authenticated {
            debug!("Ignoring input from unauthenticated player {}", ev.client_id);
            continue;
        }

        let Some((_, mut transform)) = player_transforms
            .iter_mut()
            .find(|(np, _)| np.client_id == ev.client_id)
        else {
            warn!("No ECS entity found for authenticated player {}", ev.client_id);
            continue;
        };

        let mut movement_direction = Vec3::ZERO;
        let delta_seconds = ev.input.delta_ms as f32 / 1000.0;

        let forward = ev.input.camera.forward().as_vec3();
        let right = ev.input.camera.right().as_vec3();
        let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right_horizontal = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        for action in &ev.input.inputs {
            match action {
                TransmittableAction::MoveForward => movement_direction += forward_horizontal,
                TransmittableAction::MoveBackward => movement_direction -= forward_horizontal,
                TransmittableAction::MoveRight => movement_direction += right_horizontal,
                TransmittableAction::MoveLeft => movement_direction -= right_horizontal,
                TransmittableAction::JumpOrFlyUp => {
                    if player.is_flying {
                        movement_direction += Vec3::Y;
                    }
                }
                TransmittableAction::CrouchOrFlyDown => {
                    if player.is_flying {
                        movement_direction -= Vec3::Y;
                    }
                }
                TransmittableAction::ToggleFlyMode => {
                    player.is_flying = !player.is_flying;
                }
                TransmittableAction::Hit | TransmittableAction::Modify => {}
            }
        }

        if movement_direction.length_squared() > 0.0 {
            movement_direction = movement_direction.normalize();
            let speed = if player.is_flying { FLY_SPEED } else { WALK_SPEED };
            transform.translation += movement_direction * speed * delta_seconds;
        }

        transform.rotation = ev.input.camera.rotation;
        player.last_input_processed = ev.input.time_ms;
    }
}

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

        let update = ServerPlayerUpdate {
            id: player.id,
            position,
            orientation,
            last_ack_time: player.last_input_processed,
            inventory: Box::new([]), // TODO: Implement inventory sync
        };

        server.broadcast_game_message(ServerToClientMessage::PlayerUpdate(update));
    }
}
