//! Server network dispatcher - handles incoming messages and server events
//!
//! Handles player connections, input processing, and coordinates chunk synchronization.

use crate::network::block_interactions::{handle_block_interactions, BlockInteractionEvent};
use crate::network::broadcast_world::ChunkBroadcastPlugin;
use crate::network::players::{
    broadcast_player_updates_system, handle_player_inputs_system, PlayerInputsEvent,
    PlayerRegistry,
};
use bevy::log::info;
use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer, ServerEvent};
use shared::messages::ClientToServerMessage;

use super::extensions::SendGameMessageExtension;

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

        // Server event handling (connections/disconnections)
        app.add_systems(Update, handle_server_events);

        // Message processing
        app.add_systems(Update, receive_client_messages);

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
    mut server_events: MessageReader<ServerEvent>,
    mut registry: ResMut<PlayerRegistry>,
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
            }
        }
    }
}

/// Receive and dispatch messages from connected clients
fn receive_client_messages(
    mut server: ResMut<RenetServer>,
    mut ev_player_inputs: MessageWriter<PlayerInputsEvent>,
    mut ev_block_interaction: MessageWriter<BlockInteractionEvent>,
) {
    for client_id in server.clients_id() {
        while let Some(Ok(message)) = server.receive_game_message(client_id) {
            handle_client_message(client_id, message, &mut ev_player_inputs, &mut ev_block_interaction);
        }
    }
}

/// Handle a single client message
fn handle_client_message(
    client_id: ClientId,
    message: ClientToServerMessage,
    ev_player_inputs: &mut MessageWriter<PlayerInputsEvent>,
    ev_block_interaction: &mut MessageWriter<BlockInteractionEvent>,
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
        ClientToServerMessage::AuthRegisterRequest(req) => {
            info!("Auth request from {}: {:?}", client_id, req);
            // TODO: Handle authentication properly
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
            // TODO: Handle graceful disconnection
        }
    }
}
