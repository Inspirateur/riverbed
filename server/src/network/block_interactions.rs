//! Handles block interactions from clients.
//!
//! This module processes block break/place requests from clients,
//! validates them, applies them to the VoxelWorld, and ensures
//! the changes are broadcast to all connected clients.

use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use shared::block::Block;
use shared::messages::BlockInteraction;
use shared::world::pos::pos3d::BlockPos;

use crate::network::players::PlayerRegistry;
use crate::world::voxel_world::VoxelWorld;

/// Event fired when a block interaction is received from a client.
#[derive(Message, Debug)]
pub struct BlockInteractionEvent {
    pub client_id: ClientId,
    pub interaction: BlockInteraction,
}

/// Maximum distance a player can interact with blocks
const MAX_INTERACTION_DISTANCE: f32 = 10.0;

/// Process block interactions from clients.
/// Validates the interaction, applies it to the world, and the chunk change
/// will be picked up by the broadcast system automatically.
pub fn handle_block_interactions(
    mut events: MessageReader<BlockInteractionEvent>,
    world: Res<VoxelWorld>,
    registry: Res<PlayerRegistry>,
) {
    for event in events.read() {
        // Validate the player exists
        let Some(player) = registry.get_player(event.client_id) else {
            warn!(
                "Block interaction from unknown player {}",
                event.client_id
            );
            continue;
        };

        let block_pos = event.interaction.pos;
        let new_block = event.interaction.new_block;

        // Validate distance (prevent cheating)
        let block_center = Vec3::new(
            block_pos.x as f32 + 0.5,
            block_pos.y as f32 + 0.5,
            block_pos.z as f32 + 0.5,
        );
        let distance = player.position.distance(block_center);
        
        if distance > MAX_INTERACTION_DISTANCE {
            warn!(
                "Player {} tried to interact with block at {:?} from distance {:.1} (max: {})",
                event.client_id, block_pos, distance, MAX_INTERACTION_DISTANCE
            );
            continue;
        }

        // Validate realm matches
        // TODO: Add realm check when player tracking includes realm

        // Apply the block change
        // VoxelWorld.set_block will notify the chunk_changes channel
        // which will cause the ChunkSendTracker to invalidate and re-send
        if world.set_block_safe(block_pos, new_block) {
            debug!(
                "Player {} set block at {:?} to {:?}",
                event.client_id, block_pos, new_block
            );
        } else {
            warn!(
                "Player {} failed to set block at {:?} (out of bounds)",
                event.client_id, block_pos
            );
        }
    }
}
