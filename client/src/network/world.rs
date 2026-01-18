use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::logging::logging::LogEvent;
use shared::messages::{
    ServerToClientItemStackUpdate, ServerToClientMessage, ServerToClientPlayerSpawn,
    ServerToClientPlayerUpdate,
};
use shared::STC_AUTH_CHANNEL;

use crate::network::models::client_chunk::ClientChunk;
use crate::render::MeshOrderSender;
use crate::world::ClientWorldMap;

use super::SendGameMessageExtension;

pub fn update_world_from_network(
    client: &mut ResMut<RenetClient>,
    world_map: Option<Res<ClientWorldMap>>,
    mesh_order_sender: Option<Res<MeshOrderSender>>,
    ev_player_spawn: &mut MessageWriter<ServerToClientPlayerSpawn>,
    ev_item_stacks_update: &mut MessageWriter<ServerToClientItemStackUpdate>,
    ev_player_update: &mut MessageWriter<ServerToClientPlayerUpdate>,
    ev_log_events: &mut MessageWriter<LogEvent>,
) {
    while let Some(Ok(message)) = client.receive_game_message_except_channel(STC_AUTH_CHANNEL) {
        match message {
            ServerToClientMessage::WorldUpdate(world_update) => {
                let chunk_count = world_update.new_map.len();

                if chunk_count > 0 {
                    if let (Some(world_map), Some(mesh_sender)) = (&world_map, &mesh_order_sender) {
                        for (chunk_position, chunk) in world_update.new_map {
                            let client_chunk = ClientChunk::from(chunk);
                            world_map.insert_chunk(chunk_position, client_chunk);

                            if mesh_sender.0.send(chunk_position).is_err() {
                                warn!("Failed to send mesh order for chunk {:?}", chunk_position);
                            }
                        }

                        debug!("Received and processed {} chunks from server", chunk_count);
                    } else {
                        debug!("Received {} chunks but world map not ready", chunk_count);
                    }
                }

                ev_item_stacks_update.write_batch(world_update.item_stacks);
            }
            ServerToClientMessage::PlayerSpawn(spawn_event) => {
                info!("Received spawn event {:?}", spawn_event);
                ev_player_spawn.write(spawn_event);
            }
            ServerToClientMessage::PlayerUpdate(update) => {
                ev_player_update.write(update);
            }
            ServerToClientMessage::AuthResponse(_) => {}
            ServerToClientMessage::LogEvents(events) => {
                ev_log_events.write_batch(events);
            }
        }
    }
}
