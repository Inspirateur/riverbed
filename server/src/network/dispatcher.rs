//! Server network dispatcher - handles incoming messages and server events
//! 
//! This is a minimal implementation focused on chunk synchronization.
//! TODO: Add full authentication, chat, player management as needed.

use crate::network::broadcast_world::ChunkBroadcastPlugin;
use bevy::prelude::*;
use bevy::log::{debug, info};
use bevy_renet::renet::{RenetServer, ServerEvent};

/// Plugin that sets up the core server network functionality
pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkBroadcastPlugin);
        app.add_systems(Update, handle_server_events);
    }
}

/// Handle basic server events (connections/disconnections)
fn handle_server_events(
    mut server_events: EventReader<ServerEvent>,
    _server: ResMut<RenetServer>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Player {} connected.", client_id);
                // TODO: Handle authentication, send initial world state
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player {} disconnected: {}", client_id, reason);
                // TODO: Cleanup player data, notify other players
            }
        }
    }
}
