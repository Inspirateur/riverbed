//! Common server initialization logic shared between standalone and embedded modes.

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::info;
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::RenetServer;
use bevy_renet::RenetServerPlugin;
use crossbeam::channel;
use shared::world::pos::pos3d::ChunkPos;
use shared::world::world_rng::WorldRng;
use shared::world::WorldSeed;
use shared::{get_shared_renet_config, GameServerConfig, PROTOCOL_ID, TICKS_PER_SECOND};

use crate::logging::{LogBroadcastPlugin, LogEventSender};
use crate::network::broadcast_world::ChunkChangesReceiver;
use crate::network::dispatcher::ServerNetworkPlugin;
use crate::world::voxel_world::VoxelWorld;
use crate::world::TerrainLoadPlugin;

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
pub fn setup_netcode(
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

/// Configuration options for server initialization
pub struct ServerInitConfig {
    /// The game server configuration (world name, render distance, etc.)
    pub game_config: GameServerConfig,
    /// Whether to add the log plugin (should be false when embedded in client)
    pub add_log_plugin: bool,
    /// Whether to add the log broadcast plugin (sends log events to clients)
    pub add_log_broadcast: bool,
}

/// Adds all common server plugins and resources to the app.
/// 
/// This is the shared initialization logic used by both standalone and embedded modes.
/// The caller is responsible for:
/// 1. Binding the UDP socket
/// 2. Setting up netcode (call `setup_netcode`)
/// 3. Creating the App
/// 4. Calling `app.run()`
pub fn configure_server_app(
    app: &mut App,
    server: RenetServer,
    transport: NetcodeServerTransport,
    config: ServerInitConfig,
) {
    let seed: u64 = 42; // TODO: Load from world save or generate randomly

    // Create chunk changes channel for VoxelWorld
    let (chunk_changes_tx, chunk_changes_rx) = channel::unbounded::<ChunkPos>();
    let mut voxel_world = VoxelWorld::new(chunk_changes_tx);
    voxel_world.render_distance = config.game_config.broadcast_render_distance as u32;

    // Minimal plugins for headless server
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / TICKS_PER_SECOND as f64,
        ))),
    );

    // Optionally add log plugin (standalone server needs it, embedded doesn't)
    if config.add_log_plugin {
        use shared::logging::logging::RiverbedLogPlugin;
        app.add_plugins(RiverbedLogPlugin);
    }

    // Networking plugins
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);

    // Always insert LogEventSender (needed by terrain thread), but only broadcast when configured
    if config.add_log_broadcast {
        app.add_plugins(LogBroadcastPlugin);
    } else {
        // Insert a dummy sender that goes nowhere - terrain thread needs it
        let (sender, _receiver) = crossbeam::channel::unbounded();
        app.insert_resource(LogEventSender(sender));
    }

    // Insert network resources
    app.insert_resource(server);
    app.insert_resource(transport);

    // Insert game resources
    app.insert_resource(config.game_config);
    app.insert_resource(voxel_world);
    app.insert_resource(ChunkChangesReceiver(chunk_changes_rx));
    app.insert_resource(WorldSeed(seed as u32));
    app.insert_resource(WorldRng::new(seed));

    // Add server network plugin (handles connections, auth, chunk broadcasting)
    app.add_plugins(ServerNetworkPlugin);

    // Add terrain loading plugin (handles terrain generation based on player positions)
    app.add_plugins(TerrainLoadPlugin);

    info!("Server initialized");
}
