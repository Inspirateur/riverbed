use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use shared::messages::ClientBlockInteraction;

use crate::network::dispatcher::NetworkPlayer;
use crate::network::players::PlayerRegistry;
use crate::world::voxel_world::VoxelWorld;

#[derive(Message, Debug)]
pub struct BlockInteractionEvent {
    pub client_id: ClientId,
    pub interaction: ClientBlockInteraction,
}

const MAX_INTERACTION_DISTANCE: f32 = 10.0;

pub fn handle_block_interactions(
    mut events: MessageReader<BlockInteractionEvent>,
    world: Res<VoxelWorld>,
    registry: Res<PlayerRegistry>,
    player_transforms: Query<(&NetworkPlayer, &Transform)>,
) {
    for event in events.read() {
        if !registry.is_authenticated(event.client_id) {
            warn!(
                "Block interaction from unauthenticated player {}",
                event.client_id
            );
            continue;
        }

        let Some((_, transform)) = player_transforms
            .iter()
            .find(|(np, _)| np.client_id == event.client_id)
        else {
            warn!(
                "Block interaction from player {} with no ECS entity",
                event.client_id
            );
            continue;
        };

        let block_position = event.interaction.position;
        let new_block = event.interaction.new_block;

        // Validate distance
        let block_center = Vec3::new(
            block_position.x as f32 + 0.5,
            block_position.y as f32 + 0.5,
            block_position.z as f32 + 0.5,
        );
        let player_position = transform.translation;
        let distance = player_position.distance(block_center);
        
        if distance > MAX_INTERACTION_DISTANCE {
            warn!(
                "Player {} tried to interact with block at {:?} from distance {:.1} (max: {})",
                event.client_id, block_position, distance, MAX_INTERACTION_DISTANCE
            );
            continue;
        }

        if world.set_block_safe(block_position, new_block) {
            debug!(
                "Player {} set block at {:?} to {:?}",
                event.client_id, block_position, new_block
            );
        } else {
            warn!(
                "Player {} failed to set block at {:?} (out of bounds)",
                event.client_id, block_position
            );
        }
    }
}
