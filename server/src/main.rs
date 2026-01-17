use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::{info, LogPlugin};
use bevy::prelude::*;
use bevy_renet::netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::RenetServer;
use bevy_renet::RenetServerPlugin;
use clap::Parser;
use crossbeam::channel;
use shared::world::pos::pos3d::ChunkPos;
use shared::world::WorldSeed;
use shared::{get_shared_renet_config, GameServerConfig, PROTOCOL_ID, RENDER_DISTANCE, TICKS_PER_SECOND};

use crate::network::dispatcher::ServerNetworkPlugin;
use crate::world::voxel_world::VoxelWorld;

mod generation;
mod logging;
mod network;
pub mod world;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8000)]
    port: u16,

    #[arg(short, long, default_value = "default")]
    world: String,

    #[arg(short, long, default_value_t = RENDER_DISTANCE)]
    render_distance: i32,
}

fn main() {
    let args = Args::parse();

    // Validate render_distance
    if args.render_distance < 1 || args.render_distance > 32 {
        eprintln!("Error: render_distance must be between 1 and 32");
        std::process::exit(1);
    }

    // Bind UDP socket
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), args.port);
    let socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind socket on port {}: {}", args.port, e);
            std::process::exit(1);
        }
    };

    let local_addr = socket.local_addr().expect("Failed to get local address");
    
    // Setup netcode transport
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX_EPOCH");
    
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![local_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create netcode transport");
    let server = RenetServer::new(get_shared_renet_config());

    // Create chunk changes channel for VoxelWorld
    let (chunk_changes_tx, chunk_changes_rx) = channel::unbounded::<ChunkPos>();
    let voxel_world = VoxelWorld::new(chunk_changes_tx);

    // Build the Bevy app
    let mut app = App::new();
    
    // Minimal plugins for headless server
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / TICKS_PER_SECOND as f64,
        ))),
    );
    app.add_plugins(LogPlugin::default());
    
    // Networking plugins
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);
    
    // Insert network resources
    app.insert_resource(server);
    app.insert_resource(transport);
    
    // Insert game resources
    app.insert_resource(GameServerConfig {
        world_name: args.world.clone(),
        is_solo: false,
        broadcast_render_distance: args.render_distance,
    });
    app.insert_resource(voxel_world);
    app.insert_resource(network::broadcast_world::ChunkChangesReceiver(chunk_changes_rx));
    app.insert_resource(WorldSeed(42)); // TODO: Load from world save or generate
    
    // Add server network plugin (handles chunk broadcasting)
    app.add_plugins(ServerNetworkPlugin);
    
    info!("Server starting on {}", local_addr);
    
    app.run();
}
