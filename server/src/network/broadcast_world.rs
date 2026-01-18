use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use crossbeam::channel::Receiver;
use shared::messages::{ServerToClientMessage, ServerWorldUpdate};
use shared::net::clock;
use shared::world::chunk::Chunk;
use shared::world::pos::pos2d::{chunks_in_col, ColPos};
use shared::world::pos::pos3d::ChunkPos;
use shared::world::realm::Realm;
use std::collections::{HashMap, HashSet};

use crate::network::dispatcher::NetworkPlayer;
use crate::network::players::{ClientPredictedPosition, PlayerRegistry};
use crate::world::voxel_world::VoxelWorld;

use super::extensions::SendGameMessageExtension;

const MAX_CHUNKS_PER_CLIENT_PER_TICK: usize = 16;

#[derive(Resource, Default)]
pub struct ServerTick(pub u64);

#[derive(Resource, Default)]
pub struct ChunkSendTracker {
    sent_chunks: HashMap<ClientId, HashSet<ChunkPos>>,
}

impl ChunkSendTracker {
    pub fn mark_sent(&mut self, client_id: ClientId, chunk_position: ChunkPos) {
        self.sent_chunks
            .entry(client_id)
            .or_default()
            .insert(chunk_position);
    }

    pub fn was_sent(&self, client_id: ClientId, chunk_position: &ChunkPos) -> bool {
        self.sent_chunks
            .get(&client_id)
            .map(|chunks| chunks.contains(chunk_position))
            .unwrap_or(false)
    }

    pub fn invalidate_chunk(&mut self, chunk_position: &ChunkPos) {
        for chunks in self.sent_chunks.values_mut() {
            chunks.remove(chunk_position);
        }
    }

    pub fn remove_client(&mut self, client_id: ClientId) {
        self.sent_chunks.remove(&client_id);
    }
}

#[derive(Resource)]
pub struct ChunkChangesReceiver(pub Receiver<ChunkPos>);

pub fn process_chunk_changes(
    chunk_changes: Option<Res<ChunkChangesReceiver>>,
    mut tracker: ResMut<ChunkSendTracker>,
) {
    let Some(chunk_changes) = chunk_changes else {
        return;
    };

    while let Ok(chunk_position) = chunk_changes.0.try_recv() {
        tracker.invalidate_chunk(&chunk_position);
    }
}

pub fn broadcast_world_state(
    mut server: ResMut<RenetServer>,
    mut tick: ResMut<ServerTick>,
    world: Res<VoxelWorld>,
    mut tracker: ResMut<ChunkSendTracker>,
    registry: Res<PlayerRegistry>,
    player_query: Query<(&NetworkPlayer, &ClientPredictedPosition, &Realm)>,
) {
    tick.0 += 1;
    let render_distance = world.render_distance as i32;

    for client_id in server.clients_id() {
        if !registry.is_authenticated(client_id) {
            continue;
        }

        let Some((_, predicted_pos, realm)) = player_query
            .iter()
            .find(|(np, _, _)| np.client_id == client_id)
        else {
            continue;
        };

        // Use the client's predicted position for chunk streaming.
        // This ensures we send chunks to where the client thinks it is,
        // not where the server's authoritative simulation says it is.
        let player_position = predicted_pos.0;

        let player_chunk_column = ColPos {
            x: (player_position.x / 16.0).floor() as i32,
            z: (player_position.z / 16.0).floor() as i32,
            realm: *realm,
        };

        let mut chunks_to_send: HashMap<ChunkPos, Chunk> = HashMap::new();

        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let column = ColPos {
                    x: player_chunk_column.x + dx,
                    z: player_chunk_column.z + dz,
                    realm: player_chunk_column.realm,
                };

                for chunk_position in chunks_in_col(&column) {
                    if tracker.was_sent(client_id, &chunk_position) {
                        continue;
                    }

                    let Some(chunk_entry) = world.chunks.get(&chunk_position) else {
                        continue;
                    };

                    let chunk = chunk_entry.value().read().clone();
                    chunks_to_send.insert(chunk_position, chunk);
                    tracker.mark_sent(client_id, chunk_position);

                    if chunks_to_send.len() >= MAX_CHUNKS_PER_CLIENT_PER_TICK {
                        break;
                    }
                }

                if chunks_to_send.len() >= MAX_CHUNKS_PER_CLIENT_PER_TICK {
                    break;
                }
            }

            if chunks_to_send.len() >= MAX_CHUNKS_PER_CLIENT_PER_TICK {
                break;
            }
        }

        if chunks_to_send.is_empty() {
            continue;
        }

        let message = ServerWorldUpdate {
            tick: tick.0,
            time: clock::now_ms(),
            new_map: chunks_to_send,
            item_stacks: vec![],
        };

        debug!(
            "Sending {} chunks to client {}",
            message.new_map.len(),
            client_id
        );

        server.send_game_message(client_id, ServerToClientMessage::WorldUpdate(message));
    }
}

pub struct ChunkBroadcastPlugin;

impl Plugin for ChunkBroadcastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkSendTracker>()
            .init_resource::<ServerTick>()
            .add_systems(
                Update,
                (process_chunk_changes, broadcast_world_state).chain(),
            );
    }
}
