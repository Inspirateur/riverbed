//! Server network dispatcher - handles incoming messages and server events
//!
//! Handles player connections, input processing, and coordinates chunk synchronization.

use crate::network::block_interactions::{handle_block_interactions, BlockInteractionEvent};
use crate::network::broadcast_world::{ChunkBroadcastPlugin, ChunkSendTracker, ServerTick};
use crate::network::players::{
    broadcast_player_updates_system, handle_player_inputs_system, PlayerInputsEvent,
    PlayerRegistry,
};
use bevy::log::info;
use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer, ServerEvent};
use shared::messages::{
    AuthRegisterRequest, AuthRegisterResponse, ClientToServerMessage, PlayerSave,
    PlayerSpawnEvent, ServerToClientMessage,
};
use shared::world::WorldSeed;

use super::extensions::SendGameMessageExtension;

/// Event fired when a client sends an authentication request
#[derive(Message, Debug)]
pub struct AuthRegisterEvent {
    pub client_id: ClientId,
    pub request: AuthRegisterRequest,
}

/// Plugin that sets up the core server network functionality
pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        // Add chunk broadcasting
        app.add_plugins(ChunkBroadcastPlugin);

        // Initialize player registry
        app.init_resource::<PlayerRegistry>();
        app.add_message::<PlayerInputsEvent>();
        app.add_message::<BlockInteractionEvent>();
        app.add_message::<AuthRegisterEvent>();

        // Server event handling (connections/disconnections)
        app.add_systems(Update, handle_server_events);

        // Message processing
        app.add_systems(Update, receive_client_messages);

        // Authentication handling (needs access to multiple resources)
        app.add_systems(Update, handle_auth_requests);

        // Player input processing and state broadcasting
        app.add_systems(
            Update,
            (handle_player_inputs_system, broadcast_player_updates_system).chain(),
        );

        // Block interaction processing
        app.add_systems(Update, handle_block_interactions);
    }
}

/// Handle basic server events (connections/disconnections)
fn handle_server_events(
    mut server_events: MessageReader<ServerEvent>,
    mut registry: ResMut<PlayerRegistry>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Player {} connected.", client_id);
                registry.add_player(*client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player {} disconnected: {}", client_id, reason);
                registry.remove_player(*client_id);
            }
        }
    }
}

/// Receive and dispatch messages from connected clients
fn receive_client_messages(
    mut server: ResMut<RenetServer>,
    mut ev_player_inputs: MessageWriter<PlayerInputsEvent>,
    mut ev_block_interaction: MessageWriter<BlockInteractionEvent>,
    mut ev_auth: MessageWriter<AuthRegisterEvent>,
) {
    for client_id in server.clients_id() {
        while let Some(Ok(message)) = server.receive_game_message(client_id) {
            handle_client_message(
                client_id,
                message,
                &mut ev_player_inputs,
                &mut ev_block_interaction,
                &mut ev_auth,
            );
        }
    }
}

/// Handle a single client message
fn handle_client_message(
    client_id: ClientId,
    message: ClientToServerMessage,
    ev_player_inputs: &mut MessageWriter<PlayerInputsEvent>,
    ev_block_interaction: &mut MessageWriter<BlockInteractionEvent>,
    ev_auth: &mut MessageWriter<AuthRegisterEvent>,
) {
    match message {
        ClientToServerMessage::PlayerInputs(inputs) => {
            for input in inputs {
                ev_player_inputs.write(PlayerInputsEvent { client_id, input });
            }
        }
        ClientToServerMessage::BlockInteraction(interaction) => {
            ev_block_interaction.write(BlockInteractionEvent {
                client_id,
                interaction,
            });
        }
        ClientToServerMessage::AuthRegisterRequest(request) => {
            info!("Auth request from {}: {:?}", client_id, request);
            ev_auth.write(AuthRegisterEvent { client_id, request });
        }
        ClientToServerMessage::ChatMessage(msg) => {
            info!("Chat from {}: {:?}", client_id, msg);
            // TODO: Broadcast chat messages
        }
        ClientToServerMessage::SaveWorldRequest => {
            info!("Save request from {}", client_id);
            // TODO: Handle save requests
        }
        ClientToServerMessage::Exit => {
            info!("Exit request from {}", client_id);
            // TODO: Handle graceful disconnection
        }
    }
}

/// Handle authentication requests from clients.
/// Sends back AuthRegisterResponse with session info and broadcasts player spawn to all clients.
fn handle_auth_requests(
    mut ev_auth: MessageReader<AuthRegisterEvent>,
    mut server: ResMut<RenetServer>,
    mut registry: ResMut<PlayerRegistry>,
    mut chunk_tracker: ResMut<ChunkSendTracker>,
    tick: Res<ServerTick>,
    world_seed: Res<WorldSeed>,
) {
    for event in ev_auth.read() {
        let client_id = event.client_id;
        let username = event.request.username.clone();

        // Check if username is already taken by another player
        let username_taken = registry
            .players
            .values()
            .any(|p| p.name == username && p.id != client_id);

        if username_taken {
            warn!(
                "Username '{}' already taken, rejecting auth from {}",
                username, client_id
            );
            // TODO: Send rejection message
            continue;
        }

        // Update the player's name in the registry
        if let Some(player) = registry.get_player_mut(client_id) {
            player.name = username.clone();
            player.is_authenticated = true;
            info!("Player {} authenticated as '{}'", client_id, username);
        } else {
            warn!(
                "Auth request from unknown client {}, adding to registry",
                client_id
            );
            registry.add_player(client_id);
            if let Some(player) = registry.get_player_mut(client_id) {
                player.name = username.clone();
                player.is_authenticated = true;
            }
        }

        // Reset chunk tracking for this client (they may be reconnecting)
        chunk_tracker.remove_client(client_id);

        // Build list of all player spawn events (including the new player)
        let all_player_spawns: Vec<PlayerSpawnEvent> = registry
            .players
            .values()
            .filter(|p| p.is_authenticated)
            .map(|p| PlayerSpawnEvent {
                id: p.id,
                name: p.name.clone(),
                data: PlayerSave {
                    position: p.position,
                    camera_transform: Transform::from_rotation(p.orientation),
                    is_flying: p.is_flying,
                },
            })
            .collect();

        // Get current timestamp
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Send auth response to the client
        let auth_response = AuthRegisterResponse {
            username: username.clone(),
            session_token: client_id,
            tick: tick.0,
            timestamp_ms,
            players: all_player_spawns.clone(),
            world_seed: world_seed.0,
        };

        info!(
            "Sending auth response to {} with {} players, seed={}",
            client_id,
            auth_response.players.len(),
            world_seed.0
        );
        server.send_game_message(client_id, auth_response.into());

        // Broadcast spawn event for the new player to all OTHER connected clients
        let new_player = registry.get_player(client_id).unwrap();
        let spawn_event = PlayerSpawnEvent {
            id: new_player.id,
            name: new_player.name.clone(),
            data: PlayerSave {
                position: new_player.position,
                camera_transform: Transform::from_rotation(new_player.orientation),
                is_flying: new_player.is_flying,
            },
        };

        // Send to all clients except the one who just authenticated
        for other_client_id in server.clients_id() {
            if other_client_id != client_id {
                server.send_game_message(
                    other_client_id,
                    ServerToClientMessage::PlayerSpawn(spawn_event.clone()),
                );
            }
        }

        info!(
            "Broadcasted spawn for player '{}' ({}) to other clients",
            username, client_id
        );
    }
}
