use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::{
    ItemStackUpdateEvent, PlayerSpawnEvent, PlayerUpdateEvent,
    ServerToClientMessage,
};
use shared::STC_AUTH_CHANNEL;

use crate::network::models::client_chunk::ClientChunk;
use crate::render::MeshOrderSender;
use crate::world::ClientWorldMap;

use super::SendGameMessageExtension;

/// Process incoming network messages from the server.
/// 
/// Handles world updates (chunks) and player-related events.
pub fn update_world_from_network(
    client: &mut ResMut<RenetClient>,
    world_map: Option<Res<ClientWorldMap>>,
    mesh_order_sender: Option<Res<MeshOrderSender>>,
    ev_player_spawn: &mut MessageWriter<PlayerSpawnEvent>,
    ev_item_stacks_update: &mut MessageWriter<ItemStackUpdateEvent>,
    ev_player_update: &mut MessageWriter<PlayerUpdateEvent>,
) {
    while let Some(Ok(msg)) = client.receive_game_message_except_channel(STC_AUTH_CHANNEL) {
        match msg {
            ServerToClientMessage::WorldUpdate(world_update) => {
                let chunk_count = world_update.new_map.len();
                
                if chunk_count > 0 {
                    if let (Some(world_map), Some(mesh_sender)) = (&world_map, &mesh_order_sender) {
                        // Process each chunk from the update
                        for (chunk_pos, chunk) in world_update.new_map {
                            // Convert shared Chunk to ClientChunk and insert into world
                            let client_chunk = ClientChunk::from(chunk);
                            world_map.insert_chunk(chunk_pos, client_chunk);
                            
                            // Request mesh generation for this chunk
                            if mesh_sender.0.send(chunk_pos).is_err() {
                                warn!("Failed to send mesh order for chunk {:?}", chunk_pos);
                            }
                        }
                        
                        debug!(
                            "Received and processed {} chunks from server",
                            chunk_count
                        );
                    } else {
                        debug!(
                            "Received {} chunks but world map not ready",
                            chunk_count
                        );
                    }
                }

                // Process item stack updates
                ev_item_stacks_update.write_batch(world_update.item_stacks);
            }
            ServerToClientMessage::PlayerSpawn(spawn_event) => {
                info!("Received SINGLE spawn event {:?}", spawn_event);
                ev_player_spawn.write(spawn_event);
            }
            ServerToClientMessage::PlayerUpdate(update) => {
                ev_player_update.write(update);
            }
            ServerToClientMessage::AuthRegisterResponse(_) => {}
            ServerToClientMessage::ChatConversation(_) => {}
        }
    }
}
