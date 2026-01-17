use std::sync::Arc;
use std::time::Instant;

use crate::world::ClientChunk;
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::{
    mob::MobUpdateEvent, ItemStackUpdateEvent, PlayerSpawnEvent, PlayerUpdateEvent,
    ServerToClientMessage,
};
use shared::STC_AUTH_CHANNEL;

use crate::world::ClientWorldMap;

use crate::world::WorldRenderRequestUpdateEvent;

use super::SendGameMessageExtension;

pub fn update_world_from_network(
    client: &mut ResMut<RenetClient>,
    world: &mut ResMut<ClientWorldMap>,
    ev_render: &mut MessageWriter<WorldRenderRequestUpdateEvent>,
    ev_player_spawn: &mut MessageWriter<PlayerSpawnEvent>,
    ev_mob_update: &mut MessageWriter<MobUpdateEvent>,
    ev_item_stacks_update: &mut MessageWriter<ItemStackUpdateEvent>,
    ev_player_update: &mut MessageWriter<PlayerUpdateEvent>,
) {
    while let Some(Ok(msg)) = client.receive_game_message_except_channel(STC_AUTH_CHANNEL) {
        // truncate the message to 1000 characters
        // let debug_msg = format!("{:?}", msg).chars().take(1000).collect::<String>();
        // info!("Received message: {}", debug_msg);
        match msg {
            ServerToClientMessage::WorldUpdate(world_update) => {
                debug!(
                    "Received world update, {} chunks received",
                    world_update.new_map.len()
                );

                for (pos, chunk) in world_update.new_map {
                    let chunk = Arc::new(ClientChunk {
                        map: chunk.map,
                        entity: {
                            if let Some(c) = world.map.get(&pos) {
                                c.entity
                            } else {
                                None
                            }
                        },
                        last_mesh_ts: Instant::now(),
                        current_lod: shared::world::LodLevel::default(),
                    });

                    world.map.insert(pos, chunk);
                    world.mark_dirty();
                    ev_render.write(WorldRenderRequestUpdateEvent::ChunkToReload(pos));
                }

                for (id, mob) in world_update.mobs {
                    debug!("ServerMob received: {:?}", mob);
                    ev_mob_update.write(MobUpdateEvent { id, mob });
                }

                ev_item_stacks_update.write_batch(world_update.item_stacks);

                // get current time
                // client_time.0 = world_update.time;
            }
            ServerToClientMessage::PlayerSpawn(spawn_event) => {
                info!("Received SINGLE spawn event {:?}", spawn_event);
                ev_player_spawn.write(spawn_event);
            }
            ServerToClientMessage::MobUpdate(update_event) => {
                // info!("Received mob update event {:?}", update_event);
                ev_mob_update.write(update_event);
            }
            ServerToClientMessage::PlayerUpdate(update) => {
                ev_player_update.write(update);
            }
            ServerToClientMessage::AuthRegisterResponse(_) => {}
            ServerToClientMessage::ChatConversation(_) => {}
        }
    }
}
