use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use shared::messages::{
    ItemStackUpdateEvent, PlayerSpawnEvent, PlayerUpdateEvent,
    ServerToClientMessage,
};
use shared::STC_AUTH_CHANNEL;

use super::SendGameMessageExtension;

/// Process incoming network messages from the server.
/// 
/// NOTE: World chunk processing (ClientWorldMap, WorldRenderRequestUpdateEvent, etc.)
/// is currently disabled pending the VoxelWorld/ClientWorldMap migration.
/// Only player-related events are processed.
pub fn update_world_from_network(
    client: &mut ResMut<RenetClient>,
    ev_player_spawn: &mut MessageWriter<PlayerSpawnEvent>,
    ev_item_stacks_update: &mut MessageWriter<ItemStackUpdateEvent>,
    ev_player_update: &mut MessageWriter<PlayerUpdateEvent>,
) {
    while let Some(Ok(msg)) = client.receive_game_message_except_channel(STC_AUTH_CHANNEL) {
        match msg {
            ServerToClientMessage::WorldUpdate(world_update) => {
                debug!(
                    "Received world update, {} chunks received (chunk processing disabled)",
                    world_update.new_map.len()
                );

                // TODO: Process chunks once ClientWorldMap migration is complete
                // For now, just process item stack updates
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
