//! Broadcasts world state (chunks) to connected clients.
//!
//! This module handles sending chunk data to clients based on their position
//! and tracking which chunks each client has already received.

use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer};
use crossbeam::channel::Receiver;
use shared::messages::{PlayerId, ServerToClientMessage, WorldUpdate};
use shared::world::chunk::Chunk;
use shared::world::pos::pos2d::{chunks_in_col, ColPos};
use shared::world::pos::pos3d::ChunkPos;
use shared::world::realm::Realm;
use std::collections::{HashMap, HashSet};

use crate::network::players::PlayerRegistry;
use crate::world::voxel_world::VoxelWorld;

use super::extensions::SendGameMessageExtension;

/// Maximum number of chunks to send to a single client per tick.
/// This prevents network congestion while still providing reasonable load times.
const MAX_CHUNKS_PER_CLIENT_PER_TICK: usize = 16;

/// Server tick counter resource
#[derive(Resource, Default)]
pub struct ServerTick(pub u64);

/// Tracks which chunks have been sent to each client to avoid redundant sends.
#[derive(Resource, Default)]
pub struct ChunkSendTracker {
    /// Maps client ID to the set of chunk positions they've received
    sent_chunks: HashMap<PlayerId, HashSet<ChunkPos>>,
}

impl ChunkSendTracker {
    /// Mark a chunk as sent to a client
    pub fn mark_sent(&mut self, client_id: ClientId, chunk_pos: ChunkPos) {
        self.sent_chunks
            .entry(client_id)
            .or_default()
            .insert(chunk_pos);
    }

    /// Check if a chunk has been sent to a client
    pub fn was_sent(&self, client_id: ClientId, chunk_pos: &ChunkPos) -> bool {
        self.sent_chunks
            .get(&client_id)
            .map(|chunks| chunks.contains(chunk_pos))
            .unwrap_or(false)
    }

    /// Mark a chunk as needing resend to all clients (e.g., when modified)
    pub fn invalidate_chunk(&mut self, chunk_pos: &ChunkPos) {
        for chunks in self.sent_chunks.values_mut() {
            chunks.remove(chunk_pos);
        }
    }

    /// Remove tracking for a disconnected client
    pub fn remove_client(&mut self, client_id: ClientId) {
        self.sent_chunks.remove(&client_id);
    }

    /// Invalidate all chunks in a column for all clients
    pub fn invalidate_column(&mut self, col: &ColPos) {
        for chunk_pos in chunks_in_col(col) {
            self.invalidate_chunk(&chunk_pos);
        }
    }
}

/// Resource to receive chunk change notifications from the terrain/world systems
#[derive(Resource)]
pub struct ChunkChangesReceiver(pub Receiver<ChunkPos>);

/// System that processes chunk change notifications and invalidates them in the tracker.
/// This ensures modified chunks get re-sent to clients.
pub fn process_chunk_changes(
    chunk_changes: Option<Res<ChunkChangesReceiver>>,
    mut tracker: ResMut<ChunkSendTracker>,
) {
    let Some(chunk_changes) = chunk_changes else {
        return;
    };
    
    // Process all pending chunk changes
    while let Ok(chunk_pos) = chunk_changes.0.try_recv() {
        tracker.invalidate_chunk(&chunk_pos);
    }
}

/// System that broadcasts world state to all connected clients.
/// Sends chunks that are within render distance and haven't been sent yet.
pub fn broadcast_world_state(
    mut server: ResMut<RenetServer>,
    mut tick: ResMut<ServerTick>,
    world: Res<VoxelWorld>,
    mut tracker: ResMut<ChunkSendTracker>,
    registry: Res<PlayerRegistry>,
) {
    tick.0 += 1;
    let render_distance = world.render_distance as i32;

    for client_id in server.clients_id() {
        // Get player position from registry
        let Some(player_pos) = registry.get_player_position(client_id) else {
            continue;
        };

        // Convert player position to chunk column position
        let player_chunk_col = ColPos {
            x: (player_pos.x / 16.0).floor() as i32,
            z: (player_pos.z / 16.0).floor() as i32,
            realm: Realm::Overworld, // TODO: Get actual realm from player
        };

        let mut chunks_to_send: HashMap<ChunkPos, Chunk> = HashMap::new();

        // Iterate over chunks within render distance
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let col = ColPos {
                    x: player_chunk_col.x + dx,
                    z: player_chunk_col.z + dz,
                    realm: player_chunk_col.realm,
                };

                for chunk_pos in chunks_in_col(&col) {
                    // Skip if already sent
                    if tracker.was_sent(client_id, &chunk_pos) {
                        continue;
                    }

                    // Skip if chunk doesn't exist
                    let Some(chunk_entry) = world.chunks.get(&chunk_pos) else {
                        continue;
                    };

                    // Clone the chunk data for sending
                    let chunk = chunk_entry.value().read().clone();
                    chunks_to_send.insert(chunk_pos, chunk);
                    tracker.mark_sent(client_id, chunk_pos);

                    // Limit chunks per tick to avoid network congestion
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

        // Only send if we have chunks to send
        if chunks_to_send.is_empty() {
            continue;
        }

        let msg = WorldUpdate {
            tick: tick.0,
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            new_map: chunks_to_send,
            item_stacks: vec![], // TODO: implement item stack updates
        };

        debug!(
            "Sending {} chunks to client {}",
            msg.new_map.len(),
            client_id
        );

        server.send_game_message(client_id, ServerToClientMessage::WorldUpdate(msg));
    }
}

/// Plugin that sets up chunk broadcasting
pub struct ChunkBroadcastPlugin;

impl Plugin for ChunkBroadcastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkSendTracker>()
            .init_resource::<ServerTick>()
            .add_systems(Update, (process_chunk_changes, broadcast_world_state).chain());
    }
}
