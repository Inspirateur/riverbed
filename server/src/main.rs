use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use bevy::log::info;
use bevy::prelude::*;
use clap::Parser;
use shared::{GameServerConfig, RENDER_DISTANCE};

mod generation;
mod init;
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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), args.port);
    let socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind socket on port {}: {}", args.port, e);
            std::process::exit(1);
        }
    };

    let (server, transport, local_addr) = match init::setup_netcode(socket) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to setup server netcode: {err}");
            std::process::exit(1);
        }
    };

    info!("Server starting on {}", local_addr);

    let mut app = App::new();

    init::configure_server_app(
        &mut app,
        server,
        transport,
        init::ServerInitConfig {
            game_config: GameServerConfig {
                world_name: args.world,
                is_solo: false,
                broadcast_render_distance: args.render_distance,
            },
            add_log_plugin: true,    // Standalone server needs its own logging
            add_log_broadcast: true, // Broadcast log events to clients
        },
    );

    info!("Server entering main loop");
    app.run();
}
