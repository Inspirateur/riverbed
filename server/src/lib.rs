//! Server library crate
//!
//! This module exposes the server initialization function for use by the client
//! when running in singleplayer mode (local server).

mod generation;
mod init;
mod logging;
mod network;
pub mod world;

use std::net::{IpAddr, SocketAddr, UdpSocket};

use bevy::log::{error, info};
use bevy::prelude::*;
use shared::GameServerConfig;

pub use init::{setup_netcode, NetcodeSetupError};

/// Acquires a UDP socket bound to an ephemeral port on the given IP address.
/// Used by the client to create a socket for the local server.
pub fn acquire_local_ephemeral_udp_socket(ip: IpAddr) -> std::io::Result<UdpSocket> {
    let addr = SocketAddr::new(ip, 0); // Port 0 = ephemeral port
    UdpSocket::bind(addr)
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

    let mut app = App::new();

    init::configure_server_app(
        &mut app,
        server,
        transport,
        init::ServerInitConfig {
            game_config: config,
            add_log_plugin: false, // Client already has logging configured
            add_log_broadcast: false, // Not needed for embedded server
        },
    );

    info!("Server entering main loop");
    app.run();
}
