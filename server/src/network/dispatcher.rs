use crate::network::block_interactions::{handle_block_interactions, BlockInteractionEvent};
use crate::network::broadcast_world::{ChunkBroadcastPlugin, ChunkSendTracker, ServerTick};
use crate::network::players::{
    broadcast_player_updates_system, handle_player_inputs_system, ClientPredictedPosition,
    PlayerInputsEvent, PlayerRegistry, ServerPhysicsState,
};
use bevy::log::info;
use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer, ServerEvent};
use shared::messages::{
    AuthRegisterRequest, AuthRegisterResponse, ClientToServerMessage, PlayerSave,
    ServerPlayerSpawn, ServerToClientMessage,
};
use shared::net::clock;
use shared::physics::MovementMode;
use shared::world::realm::Realm;
use shared::world::WorldSeed;
use shared::GameServerConfig;
use shared::CTS_AUTH_CHANNEL;

use super::extensions::SendGameMessageExtension;

#[derive(Message, Debug)]
pub struct AuthRegisterEvent {
    pub client_id: ClientId,
    pub request: AuthRegisterRequest,
}

#[derive(Component)]
pub struct NetworkPlayer {
    pub client_id: ClientId,
}

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkBroadcastPlugin);

        app.init_resource::<PlayerRegistry>();
        app.add_message::<PlayerInputsEvent>();
        app.add_message::<BlockInteractionEvent>();
        app.add_message::<AuthRegisterEvent>();
        app.add_message::<PlayerExitEvent>();

        app.add_systems(Update, handle_server_events);

        app.add_systems(Update, receive_client_messages);
        app.add_systems(Update, handle_auth_requests);
        app.add_systems(Update, handle_player_exit);
        app.add_systems(
            Update,
            (handle_player_inputs_system, broadcast_player_updates_system).chain(),
        );
        app.add_systems(Update, handle_block_interactions);
    }
}

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

fn receive_client_messages(
    mut server: ResMut<RenetServer>,
    mut ev_player_inputs: MessageWriter<PlayerInputsEvent>,
    mut ev_block_interaction: MessageWriter<BlockInteractionEvent>,
    mut ev_auth: MessageWriter<AuthRegisterEvent>,
    mut ev_exit: MessageWriter<PlayerExitEvent>,
) {
    for client_id in server.clients_id() {
        // Auth channel
        while let Some(Ok(message)) =
            server.receive_game_message_by_channel(client_id, CTS_AUTH_CHANNEL)
        {
            handle_client_message(
                client_id,
                message,
                &mut ev_player_inputs,
                &mut ev_block_interaction,
                &mut ev_auth,
                &mut ev_exit,
            );
        }

        // All other channels
        while let Some(Ok(message)) =
            server.receive_game_message_except_channel(client_id, CTS_AUTH_CHANNEL)
        {
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
        ClientToServerMessage::SaveWorldRequest => {
            info!("Save request from {}", client_id);
        }
        ClientToServerMessage::Exit => {
            info!("Exit request from {}", client_id);
            ev_exit.write(PlayerExitEvent { client_id });
        }
    }
}

#[derive(Message, Debug)]
pub struct PlayerExitEvent {
    pub client_id: ClientId,
}

fn handle_auth_requests(
    mut commands: Commands,
    mut ev_auth: MessageReader<AuthRegisterEvent>,
    mut server: ResMut<RenetServer>,
    mut registry: ResMut<PlayerRegistry>,
    mut chunk_tracker: ResMut<ChunkSendTracker>,
    tick: Res<ServerTick>,
    world_seed: Res<WorldSeed>,
    existing_players: Query<(&NetworkPlayer, &Transform, Option<&ServerPhysicsState>)>,
) {
    use crate::network::players::DEFAULT_SPAWN_POSITION;

    for event in ev_auth.read() {
        let client_id = event.client_id;
        let username = event.request.username.clone();

        let username_taken = registry
            .players
            .values()
            .any(|p| p.name == username && p.id != client_id);

        if username_taken {
            warn!(
                "Username '{}' already taken, rejecting auth from {}",
                username, client_id
            );
            continue;
        }

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

        let spawn_position = DEFAULT_SPAWN_POSITION;

        commands.spawn((
            Transform::from_translation(spawn_position),
            Realm::Overworld,
            NetworkPlayer { client_id },
            ServerPhysicsState::default(),
            ClientPredictedPosition(spawn_position),
        ));
        info!(
            "Spawned ECS entity for player {} at {:?}",
            client_id, spawn_position
        );

        chunk_tracker.remove_client(client_id);

        let mut all_player_spawns: Vec<ServerPlayerSpawn> = Vec::new();

        for player in registry.players.values().filter(|p| p.is_authenticated) {
            let (position, orientation, is_flying) = if player.id == client_id {
                (spawn_position, Quat::IDENTITY, false)
            } else {
                existing_players
                    .iter()
                    .find(|(np, _, _)| np.client_id == player.id)
                    .map(|(_, transform, physics)| {
                        let is_flying = physics
                            .map(|p| p.movement_mode == MovementMode::Flying)
                            .unwrap_or(false);
                        (transform.translation, transform.rotation, is_flying)
                    })
                    .unwrap_or((DEFAULT_SPAWN_POSITION, Quat::IDENTITY, false))
            };

            all_player_spawns.push(ServerPlayerSpawn {
                id: player.id,
                name: player.name.clone(),
                data: PlayerSave {
                    position,
                    camera_transform: Transform::from_rotation(orientation),
                    is_flying,
                },
            });
        }

        let timestamp_ms = clock::now_ms();

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

        let new_player = registry.get_player(client_id).unwrap();
        let spawn_event = ServerPlayerSpawn {
            id: new_player.id,
            name: new_player.name.clone(),
            data: PlayerSave {
                position: spawn_position,
                camera_transform: Transform::default(),
                is_flying: false, // New players start in walking mode
            },
        };

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

        let player_name = registry
            .get_player(client_id)
            .map(|player| player.name.clone())
            .unwrap_or_else(|| format!("Unknown-{}", client_id));

        info!("Player '{}' ({}) is disconnecting", player_name, client_id);

        registry.remove_player(client_id);
        chunk_tracker.remove_client(client_id);
        server.disconnect(client_id);

        info!(
            "Player '{}' ({}) disconnected successfully",
            player_name, client_id
        );

        if config.is_solo {
            info!("Solo mode: shutting down server as player disconnected");
            app_exit.write(AppExit::Success);
        }
    }
}
