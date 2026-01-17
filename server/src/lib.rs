//! Server library crate
//!
//! This module exposes the server initialization function for use by the client
//! when running in singleplayer mode (local server).

mod generation;
mod logging;
mod network;
pub mod world;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::{error, info};
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::RenetServer;
use bevy_renet::RenetServerPlugin;
use crossbeam::channel;
use shared::world::pos::pos3d::ChunkPos;
use shared::world::WorldSeed;
use shared::{get_shared_renet_config, GameServerConfig, PROTOCOL_ID, TICKS_PER_SECOND};

use crate::network::broadcast_world::ChunkChangesReceiver;
use crate::network::dispatcher::ServerNetworkPlugin;
use crate::world::voxel_world::VoxelWorld;

/// Acquires a UDP socket bound to an ephemeral port on the given IP address.
/// Used by the client to create a socket for the local server.
pub fn acquire_local_ephemeral_udp_socket(ip: IpAddr) -> std::io::Result<UdpSocket> {
    let addr = SocketAddr::new(ip, 0); // Port 0 = ephemeral port
    UdpSocket::bind(addr)
}

/// Error types that can occur during server netcode setup
#[derive(Debug)]
pub enum NetcodeSetupError {
    SocketAddr(std::io::Error),
    Time(std::time::SystemTimeError),
    Transport(String),
}

impl std::fmt::Display for NetcodeSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetcodeSetupError::SocketAddr(err) => {
                write!(f, "Failed to get socket local address: {err}")
            }
            NetcodeSetupError::Time(err) => {
                write!(f, "System time error: {err}")
            }
            NetcodeSetupError::Transport(err) => {
                write!(f, "Failed to create netcode transport: {err}")
            }
        }
    }
}

/// Sets up the netcode networking layer for the server.
/// Returns the RenetServer, transport, and the address the server is bound to.
fn setup_netcode(
    socket: UdpSocket,
) -> Result<(RenetServer, NetcodeServerTransport, SocketAddr), NetcodeSetupError> {
    let local_addr = socket.local_addr().map_err(NetcodeSetupError::SocketAddr)?;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(NetcodeSetupError::Time)?;

    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![local_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .map_err(|err| NetcodeSetupError::Transport(err.to_string()))?;

    let server = RenetServer::new(get_shared_renet_config());

    Ok((server, transport, local_addr))
}

/// Initialize and run the server with the given configuration.
///
/// This function blocks until the server shuts down.
/// It's designed to be called from a separate thread when running in singleplayer mode.
///
/// # Arguments
/// * `socket` - A pre-bound UDP socket for the server to use
/// * `config` - Server configuration including world name and settings
///
/// # Example
/// ```ignore
/// use std::thread;
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let socket = server::acquire_local_ephemeral_udp_socket(
///     IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
/// ).unwrap();
/// let addr = socket.local_addr().unwrap();
///
/// thread::spawn(move || {
///     server::init(socket, GameServerConfig {
///         world_name: "my_world".to_string(),
///         is_solo: true,
///         broadcast_render_distance: 8,
///     });
/// });
/// ```
pub fn init(socket: UdpSocket, config: GameServerConfig) {
    let (server, transport, addr) = match setup_netcode(socket) {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to setup server netcode: {err}");
            return;
        }
    };

    info!("Server starting on {}", addr);

    // Create chunk changes channel for VoxelWorld
    let (chunk_changes_tx, chunk_changes_rx) = channel::unbounded::<ChunkPos>();
    let mut voxel_world = VoxelWorld::new(chunk_changes_tx);
    voxel_world.render_distance = config.broadcast_render_distance as u32;

    // Build the Bevy app
    let mut app = App::new();

    // Minimal plugins for headless server
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / TICKS_PER_SECOND as f64,
        ))),
    );
    // Note: We don't add LogPlugin here because when running embedded in the client,
    // the client already has logging configured. Adding it here would cause a conflict.

    // Networking plugins
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);

    // Insert network resources
    app.insert_resource(server);
    app.insert_resource(transport);

    // Insert game resources
    app.insert_resource(config);
    app.insert_resource(voxel_world);
    app.insert_resource(ChunkChangesReceiver(chunk_changes_rx));
    app.insert_resource(WorldSeed(42)); // TODO: Load from world save or generate randomly

    // Add server network plugin (handles connections, auth, chunk broadcasting)
    app.add_plugins(ServerNetworkPlugin);

    info!("Server initialized, entering main loop");

    app.run();
}
