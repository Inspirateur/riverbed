use bevy::prelude::*;
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::{renet::RenetClient, RenetClientPlugin};
use rand::Rng;
use shared::{
    get_shared_renet_config, GameServerConfig, STC_AUTH_CHANNEL,
    SOCKET_BIND_ERROR, TARGET_SERVER_ADDR_ERROR, NETCODE_CLIENT_TRANSPORT_ERROR, 
    UNIX_EPOCH_TIME_ERROR, RENDER_DISTANCE,
};
use shared::logging::logging::LogEvent;
use shared::net::clock;

use crate::network::world::update_world_from_network;
use crate::render::MeshOrderSender;
use crate::world::ClientWorldMap;
use shared::messages::{
    AuthRegisterRequest, ServerItemStackUpdate, PlayerId, ServerPlayerSpawn, ServerPlayerUpdate,
    ServerToClientMessage,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::time::Duration;
use std::{net::UdpSocket, time::SystemTime};

use super::SendGameMessageExtension;
use super::buffered_client::{SyncTime, SyncTimeExt};

#[derive(Resource, Debug, Default)]
pub struct PlayerNameSupplied {
    pub name: String,
}

#[derive(Resource, Debug, Clone)]
pub struct SelectedWorld {
    pub name: Option<String>,
}

impl Default for SelectedWorld {
    fn default() -> Self {
        Self {
            name: Some("default".to_string()),
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ServerTickAtConnect(pub u64);

#[derive(Resource, Default, Debug, Clone)]
pub struct WorldSeed(pub u32);

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TargetServerState {
    #[default]
    Initial,
    Establishing,
    FullyReady,
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

#[derive(Resource, Debug, Clone, Default)]
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

    app.add_plugins(NetcodeClientPlugin);

    // Only insert TargetServer if not already present (may be set by CLI arg)
    app.init_resource::<TargetServer>();
}

pub fn launch_local_server_system(
    mut target: ResMut<TargetServer>,
    selected_world: Res<SelectedWorld>,
) {
    if target.address.is_some() {
        debug!("Skipping launch local server - address already set");
        return;
    }

    if let Some(world_name) = &selected_world.name {
        info!("Launching local server with world: {}", world_name);

        let socket = match server::acquire_local_ephemeral_udp_socket(IpAddr::V4(Ipv4Addr::new(
            127, 0, 0, 1,
        ))) {
            Ok(socket) => socket,
            Err(err) => {
                error!("{}: {err}", SOCKET_BIND_ERROR);
                return;
            }
        };

        let address = match socket.local_addr() {
            Ok(address) => address,
            Err(err) => {
                error!("Failed to get socket local address: {err}");
                return;
            }
        };
        info!("Local server will bind to: {}", address);

        let world_name_clone = world_name.clone();

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

        thread::sleep(Duration::from_millis(100));

        target.address = Some(address);
        info!("Local server launched, client will connect to {}", address);
    } else {
        error!("Error: No world selected. Unable to launch the server.");
    }
}

pub fn poll_network_messages(
    mut client: ResMut<RenetClient>,
    world_map: Option<Res<ClientWorldMap>>,
    mesh_order_sender: Option<Res<MeshOrderSender>>,
    mut ev_player_spawn: MessageWriter<ServerPlayerSpawn>,
    mut ev_item_stacks_update: MessageWriter<ServerItemStackUpdate>,
    mut ev_player_update: MessageWriter<ServerPlayerUpdate>,
    mut ev_log_events: MessageWriter<LogEvent>,
) {
    update_world_from_network(
        &mut client,
        world_map,
        mesh_order_sender,
        &mut ev_player_spawn,
        &mut ev_item_stacks_update,
        &mut ev_player_update,
        &mut ev_log_events,
    );
}

pub fn init_server_connection(
    mut commands: Commands,
    target: Res<TargetServer>,
    current_player_id: Res<CurrentPlayerProfile>,
) {
    let Some(address) = target.address else {
        error!("{TARGET_SERVER_ADDR_ERROR}");
        return;
    };
    let id = current_player_id.into_inner().id;
    commands.queue(move |world: &mut World| {
        world.remove_resource::<RenetClient>();
        world.remove_resource::<NetcodeClientTransport>();

        let authentication = ClientAuthentication::Unsecure {
            server_addr: address,
            client_id: id,
            user_data: None,
            protocol_id: shared::PROTOCOL_ID,
        };

        info!(
            "Attempting to connect to: {} with data {:?}",
            address, authentication
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

        info!("Network subsystem initialized");
    })
}

pub fn network_failure_handler(mut renet_error: MessageReader<NetcodeTransportError>) {
    for error in renet_error.read() {
        error!("network error: {}", error);
    }
}

pub fn establish_authenticated_connection_to_server(
    mut client: ResMut<RenetClient>,
    mut target: ResMut<TargetServer>,
    current_profile: Res<CurrentPlayerProfile>,
    mut ev_spawn: MessageWriter<ServerPlayerSpawn>,
    mut server_tick: ResMut<ServerTickAtConnect>,
    mut world_seed: ResMut<WorldSeed>,
    mut sync_time: ResMut<SyncTime>,
) {
    if target.session_token.is_some() {
        return;
    }

    if target.state == TargetServerState::Initial {
        if target.username.is_none() {
            target.username = Some(current_profile.into_inner().name.clone());
        }

        let username = target.username.as_ref().unwrap();

        let auth_request = AuthRegisterRequest {
            username: username.clone(),
        };
        info!("Sending auth request: {:?}", auth_request);
        client.send_game_message(auth_request.into());
        target.state = TargetServerState::Establishing;
    }

    while let Some(Ok(message)) = client.receive_game_message_by_channel(STC_AUTH_CHANNEL) {
        match message {
            ServerToClientMessage::AuthRegisterResponse(response) => {
                let username = response.username.clone();
                target.username = Some(response.username);
                target.session_token = Some(response.session_token);
                target.state = TargetServerState::FullyReady;
                server_tick.0 = response.tick;
                world_seed.0 = response.world_seed;

                let local_now = clock::now_ms();
                let offset_ms = clock::compute_offset(response.timestamp_ms, local_now);
                sync_time.clock.last_ms = local_now;
                sync_time.clock.curr_ms = local_now;
                sync_time.set_offset(offset_ms);

                info!("Successfully authenticated as {}", username);
                info!("Received world seed: {}", response.world_seed);
                for player in response.players {
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
