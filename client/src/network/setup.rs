use bevy::prelude::*;
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::{renet::RenetClient, RenetClientPlugin};
use rand::Rng;
use shared::{
    get_shared_renet_config, GameServerConfig, STC_AUTH_CHANNEL,
    SOCKET_BIND_ERROR, TARGET_SERVER_ADDR_ERROR, NETCODE_CLIENT_TRANSPORT_ERROR, 
    UNIX_EPOCH_TIME_ERROR, USERNAME_MISSING_AUTHENTICATED_ERROR, RENDER_DISTANCE,
};

use crate::network::world::update_world_from_network;
use crate::network::CachedChatConversation;
use crate::render::MeshOrderSender;
use crate::world::ClientWorldMap;
use shared::messages::{
    AuthRegisterRequest, ItemStackUpdateEvent, PlayerId, PlayerSpawnEvent, PlayerUpdateEvent,
    ServerToClientMessage,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::time::Duration;
use std::{net::UdpSocket, time::SystemTime};

use super::SendGameMessageExtension;

// Resource for player name input (for multiplayer)
#[derive(Resource, Debug, Default)]
pub struct PlayerNameSupplied {
    pub name: String,
}

// Resource for selected world (for world selection menu)
#[derive(Resource, Debug, Clone)]
pub struct SelectedWorld {
    pub name: Option<String>,
}

impl Default for SelectedWorld {
    fn default() -> Self {
        // Default to "default" world for singleplayer MVP
        // This will be replaced by a world selection menu later
        Self {
            name: Some("default".to_string()),
        }
    }
}

// Resource for client-side time tracking
#[derive(Resource, Default, Debug, Clone)]
pub struct ClientTime(pub u64);

// Resource for world seed
#[derive(Resource, Default, Debug, Clone)]
pub struct WorldSeed(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum TargetServerState {
    Initial,
    Establishing,
    ConnectionEstablished,
    FullyReady, // player has spawned
}

#[derive(Resource, Clone)]
pub struct CurrentPlayerProfile {
    pub id: PlayerId,
    pub name: String,
}

impl CurrentPlayerProfile {
    pub(crate) fn new() -> Self {
        let mut rng = rand::rng();
        let id: u64 = rng.random();
        Self {
            id,
            name: format!("Player-{id}"),
        }
    }
}

fn hash_string_to_u64(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

impl FromWorld for CurrentPlayerProfile {
    fn from_world(world: &mut World) -> Self {
        let player_name = world.get_resource::<PlayerNameSupplied>();
        match player_name {
            Some(player_name) => Self {
                id: hash_string_to_u64(&player_name.name),
                name: player_name.name.clone(),
            },
            None => CurrentPlayerProfile::new(),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct TargetServer {
    pub address: Option<SocketAddr>,
    pub username: Option<String>,
    pub session_token: Option<u64>,
    pub state: TargetServerState,
}

pub fn add_base_netcode(app: &mut App) {
    app.add_plugins(RenetClientPlugin);

    let client = RenetClient::new(get_shared_renet_config());
    app.insert_resource(client);

    // Setup the transport layer
    app.add_plugins(NetcodeClientPlugin);

    // TODO: change username
    app.insert_resource(TargetServer {
        address: None,
        username: None,
        session_token: None,
        state: TargetServerState::Initial,
    });
}

pub fn launch_local_server_system(
    mut target: ResMut<TargetServer>,
    selected_world: Res<SelectedWorld>,
) {
    if target.address.is_some() {
        debug!("Skipping launch local server");
        return;
    }

    if let Some(world_name) = &selected_world.name {
        info!("Launching local server with world: {}", world_name);

        // Acquire an ephemeral UDP socket for the local server
        let socket = match server::acquire_local_ephemeral_udp_socket(IpAddr::V4(Ipv4Addr::new(
            127, 0, 0, 1,
        ))) {
            Ok(socket) => socket,
            Err(err) => {
                error!("{}: {err}", SOCKET_BIND_ERROR);
                return;
            }
        };

        // Get the address the socket was bound to
        let addr = match socket.local_addr() {
            Ok(addr) => addr,
            Err(err) => {
                error!("Failed to get socket local address: {err}");
                return;
            }
        };
        info!("Local server will bind to: {}", addr);

        // Clone data needed for the server thread
        let world_name_clone = world_name.clone();

        // Spawn the server in a separate thread
        thread::spawn(move || {
            server::init(
                socket,
                GameServerConfig {
                    world_name: world_name_clone,
                    is_solo: true,
                    broadcast_render_distance: RENDER_DISTANCE,
                },
            );
        });

        // Give the server thread a moment to initialize before we try to connect
        // The socket is already bound, but the server needs to set up its Bevy app
        thread::sleep(Duration::from_millis(100));

        // Store the server address for the client to connect to
        target.address = Some(addr);
        info!("Local server launched, client will connect to {}", addr);
    } else {
        error!("Error: No world selected. Unable to launch the server.");
    }
}

pub fn poll_network_messages(
    mut client: ResMut<RenetClient>,
    world_map: Option<Res<ClientWorldMap>>,
    mesh_order_sender: Option<Res<MeshOrderSender>>,
    mut ev_player_spawn: MessageWriter<PlayerSpawnEvent>,
    mut ev_item_stacks_update: MessageWriter<ItemStackUpdateEvent>,
    mut ev_player_update: MessageWriter<PlayerUpdateEvent>,
) {
    update_world_from_network(
        &mut client,
        world_map,
        mesh_order_sender,
        &mut ev_player_spawn,
        &mut ev_item_stacks_update,
        &mut ev_player_update,
    );
}

pub fn init_server_connection(
    mut commands: Commands,
    target: Res<TargetServer>,
    current_player_id: Res<CurrentPlayerProfile>,
) {
    let Some(addr) = target.address else {
        error!("{TARGET_SERVER_ADDR_ERROR}");
        return;
    };
    let id = current_player_id.into_inner().id;
    commands.queue(move |world: &mut World| {
        world.remove_resource::<RenetClient>();
        world.remove_resource::<NetcodeClientTransport>();
        world.remove_resource::<CachedChatConversation>();

        let authentication = ClientAuthentication::Unsecure {
            server_addr: addr,
            client_id: id,
            user_data: None,
            protocol_id: shared::PROTOCOL_ID,
        };

        info!(
            "Attempting to connect to: {} with data {:?}",
            addr, authentication
        );

        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => socket,
            Err(err) => {
                error!("{}: {err}", SOCKET_BIND_ERROR);
                return;
            }
        };
        let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(time) => time,
            Err(err) => {
                error!("{}: {err}", UNIX_EPOCH_TIME_ERROR);
                return;
            }
        };
        let transport = match NetcodeClientTransport::new(current_time, authentication, socket) {
            Ok(transport) => transport,
            Err(err) => {
                error!("{}: {err}", NETCODE_CLIENT_TRANSPORT_ERROR);
                return;
            }
        };

        let client = RenetClient::new(get_shared_renet_config());
        world.insert_resource(client);
        world.insert_resource(transport);

        world.insert_resource(CachedChatConversation { ..default() });

        info!("Network subsystem initialized");
    })
}

pub fn network_failure_handler(mut renet_error: MessageReader<NetcodeTransportError>) {
    for e in renet_error.read() {
        error!("network error: {}", e);
    }
}

pub fn establish_authenticated_connection_to_server(
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    current_profile: Res<CurrentPlayerProfile>,
    mut ev_spawn: MessageWriter<PlayerSpawnEvent>,
    mut client_time: ResMut<ClientTime>,
    mut world_seed: ResMut<WorldSeed>,
) {
    if target.session_token.is_some() {
        let Some(username) = target.username.as_ref() else {
            error!("{USERNAME_MISSING_AUTHENTICATED_ERROR}");
            return;
        };
        info!("Successfully acquired a session token as {}", username);
        return;
    }

    if target.state == TargetServerState::Initial {
        if target.username.is_none() {
            target.username = Some(current_profile.into_inner().name.clone());
        }

        let username = target.username.as_ref().unwrap();

        let auth_msg = AuthRegisterRequest {
            username: username.clone(),
        };
        info!("Sending auth request: {:?}", auth_msg);
        client.send_game_message(auth_msg.into());
        target.state = TargetServerState::Establishing;
    }

    while let Some(Ok(message)) = client.receive_game_message_by_channel(STC_AUTH_CHANNEL) {
        match message {
            ServerToClientMessage::AuthRegisterResponse(message) => {
                target.username = Some(message.username);
                target.session_token = Some(message.session_token);
                target.state = TargetServerState::ConnectionEstablished;
                client_time.0 = message.tick;
                world_seed.0 = message.world_seed;
                info!("Received world seed: {}", message.world_seed);
                // TODO: handle clock sync using the timestamp_ms field
                // it will become very important if the lantency is high
                for player in message.players {
                    ev_spawn.write(player);
                }
                info!("Connected! {:?}", target);
            }
            _ => {
                panic!("Unexpected message: {message:?}");
            }
        }
    }
}
