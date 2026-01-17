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
use shared::world::realm::Realm;
use shared::world::WorldSeed;
use shared::GameServerConfig;

use super::extensions::SendGameMessageExtension;

/// Event fired when a client sends an authentication request
#[derive(Message, Debug)]
pub struct AuthRegisterEvent {
    pub client_id: ClientId,
    pub request: AuthRegisterRequest,
}

/// Component that links a player entity to its network client ID
#[derive(Component)]
pub struct NetworkPlayer {
    pub client_id: ClientId,
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
        app.add_message::<PlayerExitEvent>();

        // Server event handling (connections/disconnections)
        app.add_systems(Update, handle_server_events);

        // Message processing
        app.add_systems(Update, receive_client_messages);

        // Authentication handling (needs access to multiple resources)
        app.add_systems(Update, handle_auth_requests);

        // Player exit handling
        app.add_systems(Update, handle_player_exit);

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
    mut commands: Commands,
    mut server_events: MessageReader<ServerEvent>,
    mut registry: ResMut<PlayerRegistry>,
    player_entities: Query<(Entity, &NetworkPlayer)>,
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
                
                // Despawn the player's ECS entity
                for (entity, network_player) in player_entities.iter() {
                    if network_player.client_id == *client_id {
                        commands.entity(entity).despawn();
                        info!("Despawned ECS entity for player {}", client_id);
                        break;
                    }
                }
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
    mut ev_exit: MessageWriter<PlayerExitEvent>,
) {
    for client_id in server.clients_id() {
        while let Some(Ok(message)) = server.receive_game_message(client_id) {
            handle_client_message(
                client_id,
                message,
                &mut ev_player_inputs,
                &mut ev_block_interaction,
                &mut ev_auth,
                &mut ev_exit,
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
    ev_exit: &mut MessageWriter<PlayerExitEvent>,
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
            ev_exit.write(PlayerExitEvent { client_id });
        }
    }
}

/// Event fired when a player sends an Exit message
#[derive(Message, Debug)]
pub struct PlayerExitEvent {
    pub client_id: ClientId,
}

/// Handle authentication requests from clients.
/// Sends back AuthRegisterResponse with session info and broadcasts player spawn to all clients.
/// 
/// Note: This system spawns ECS entities for new players. The Transform on these entities
/// is the single source of truth for player position. For existing players, we query their
/// ECS Transform; for the newly authenticating player, we use DEFAULT_SPAWN_POSITION since
/// their entity won't be queryable until the next frame.
fn handle_auth_requests(
    mut commands: Commands,
    mut ev_auth: MessageReader<AuthRegisterEvent>,
    mut server: ResMut<RenetServer>,
    mut registry: ResMut<PlayerRegistry>,
    mut chunk_tracker: ResMut<ChunkSendTracker>,
    tick: Res<ServerTick>,
    world_seed: Res<WorldSeed>,
    existing_players: Query<(&NetworkPlayer, &Transform)>,
) {
    use crate::network::players::DEFAULT_SPAWN_POSITION;

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

        // Spawn an ECS entity for this player with Transform and Realm
        // The Transform is the single source of truth for player position
        let spawn_pos = DEFAULT_SPAWN_POSITION;
        
        commands.spawn((
            Transform::from_translation(spawn_pos),
            Realm::Overworld,
            NetworkPlayer { client_id },
        ));
        info!(
            "Spawned ECS entity for player {} at {:?}",
            client_id, spawn_pos
        );

        // Reset chunk tracking for this client (they may be reconnecting)
        chunk_tracker.remove_client(client_id);

        // Build list of all player spawn events
        // For existing players, get position from their ECS Transform
        // For the new player (just authenticated), use the spawn position we just set
        let mut all_player_spawns: Vec<PlayerSpawnEvent> = Vec::new();
        
        for player in registry.players.values().filter(|p| p.is_authenticated) {
            let (position, orientation) = if player.id == client_id {
                // New player - use spawn position (entity not queryable yet)
                (spawn_pos, Quat::IDENTITY)
            } else {
                // Existing player - query ECS Transform
                existing_players
                    .iter()
                    .find(|(np, _)| np.client_id == player.id)
                    .map(|(_, t)| (t.translation, t.rotation))
                    .unwrap_or((DEFAULT_SPAWN_POSITION, Quat::IDENTITY))
            };

            all_player_spawns.push(PlayerSpawnEvent {
                id: player.id,
                name: player.name.clone(),
                data: PlayerSave {
                    position,
                    camera_transform: Transform::from_rotation(orientation),
                    is_flying: player.is_flying,
                },
            });
        }

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
            players: all_player_spawns,
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
                position: spawn_pos,
                camera_transform: Transform::default(),
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

/// Handle player exit requests.
/// Disconnects the player and shuts down the server if running in solo mode.
fn handle_player_exit(
    mut ev_exit: MessageReader<PlayerExitEvent>,
    mut server: ResMut<RenetServer>,
    mut registry: ResMut<PlayerRegistry>,
    mut chunk_tracker: ResMut<ChunkSendTracker>,
    config: Res<GameServerConfig>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for event in ev_exit.read() {
        let client_id = event.client_id;
        
        // Get player name for logging before removing
        let player_name = registry
            .get_player(client_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Unknown-{}", client_id));

        info!("Player '{}' ({}) is disconnecting", player_name, client_id);

        // Remove player from registry
        registry.remove_player(client_id);
        
        // Clear chunk tracking for this client
        chunk_tracker.remove_client(client_id);

        // Disconnect the client from the server
        server.disconnect(client_id);

        info!("Player '{}' ({}) disconnected successfully", player_name, client_id);

        // If running in solo mode, shut down the server when the player exits
        if config.is_solo {
            info!("Solo mode: shutting down server as player disconnected");
            app_exit.write(AppExit::Success);
        }
    }
}
