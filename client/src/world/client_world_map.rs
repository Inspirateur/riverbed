//! Client-side world map that stores chunks and provides block access.
//! 
//! This module provides a read-only view of the world for client systems,
//! with block modifications queued as events for network transmission.

use bevy::prelude::*;
use crossbeam::channel::Sender;
use crossbeam_skiplist::SkipMap;
use parking_lot::RwLock;
use shared::{
    block::Block,
    world::{
        BlockRayCastHit, MAX_HEIGHT, Y_CHUNKS,
        pos::{
            pos2d::ColPos,
            pos3d::{BlockPos, ChunkPos},
        },
        realm::Realm,
    },
};
use std::sync::Arc;
use crate::network::models::client_chunk::ClientChunk;

/// Client-side world map resource.
/// 
/// Provides read-only block access for physics, raycasting, and rendering.
/// Block modifications are queued as events rather than applied directly.
#[derive(Resource, Clone)]
pub struct ClientWorldMap {
    /// The chunk data, shared with the mesh thread
    pub chunks: Arc<SkipMap<ChunkPos, RwLock<ClientChunk>>>,
    /// Channel to notify mesh thread of chunk changes
    chunk_changes: Sender<ChunkPos>,
    /// Render distance in chunks
    pub render_distance: u32,
}

impl ClientWorldMap {
    pub fn new(chunk_changes: Sender<ChunkPos>) -> Self {
        ClientWorldMap {
            chunks: Arc::new(SkipMap::new()),
            chunk_changes,
            render_distance: 32,
        }
    }

    /// Get a block at the given position
    pub fn get_block(&self, pos: BlockPos) -> Block {
        let (chunk_pos, chunked_pos) = <(ChunkPos, _)>::from(pos);
        match self.chunks.get(&chunk_pos) {
            None => Block::Air,
            Some(chunk) => chunk.value().read().get(chunked_pos).clone(),
        }
    }

    /// Get a block at the given position, returning Air for out-of-bounds
    pub fn get_block_safe(&self, pos: BlockPos) -> Block {
        if pos.y < 0 || pos.y >= MAX_HEIGHT as i32 {
            Block::Air
        } else {
            self.get_block(pos)
        }
    }

    /// Check if a column is loaded (has any chunks)
    pub fn is_col_loaded(&self, player_pos: Vec3, realm: Realm) -> bool {
        let (chunk_pos, _): (ChunkPos, _) = <BlockPos>::from((player_pos, realm)).into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk = ChunkPos {
                x: chunk_pos.x,
                y,
                z: chunk_pos.z,
                realm: chunk_pos.realm,
            };
            if self.chunks.contains_key(&chunk) {
                return true;
            }
        }
        false
    }

    /// Perform a raycast against the world
    pub fn raycast(
        &self,
        realm: Realm,
        start: Vec3,
        dir: Vec3,
        dist: f32,
    ) -> Option<BlockRayCastHit> {
        let mut pos = BlockPos {
            realm,
            x: start.x.floor() as i32,
            y: start.y.floor() as i32,
            z: start.z.floor() as i32,
        };
        let mut last_pos;
        let sx = dir.x.signum() as i32;
        let sy = dir.y.signum() as i32;
        let sz = dir.z.signum() as i32;
        if sx == 0 && sy == 0 && sz == 0 {
            return None;
        }
        let next_x = (pos.x + sx.max(0)) as f32;
        let next_y = (pos.y + sy.max(0)) as f32;
        let next_z = (pos.z + sz.max(0)) as f32;
        let mut t_max_x = (next_x - start.x) / dir.x;
        let mut t_max_y = (next_y - start.y) / dir.y;
        let mut t_max_z = (next_z - start.z) / dir.z;
        let slope_x = 1. / dir.x.abs();
        let slope_y = 1. / dir.y.abs();
        let slope_z = 1. / dir.z.abs();
        loop {
            last_pos = pos;
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x >= dist {
                        return None;
                    };
                    pos.x += sx;
                    t_max_x += slope_x;
                } else {
                    if t_max_z >= dist {
                        return None;
                    };
                    pos.z += sz;
                    t_max_z += slope_z;
                }
            } else if t_max_y < t_max_z {
                if t_max_y >= dist {
                    return None;
                };
                pos.y += sy;
                t_max_y += slope_y;
            } else {
                if t_max_z >= dist {
                    return None;
                };
                pos.z += sz;
                t_max_z += slope_z;
            }
            if self.get_block_safe(pos).is_targetable() {
                return Some(BlockRayCastHit {
                    pos,
                    normal: Vec3 {
                        x: (last_pos.x - pos.x) as f32,
                        y: (last_pos.y - pos.y) as f32,
                        z: (last_pos.z - pos.z) as f32,
                    },
                });
            }
        }
    }

    /// Insert or update a chunk (called when receiving data from server)
    pub fn insert_chunk(&self, chunk_pos: ChunkPos, chunk: ClientChunk) {
        self.chunks.insert(chunk_pos, RwLock::new(chunk));
        let _ = self.chunk_changes.send(chunk_pos);
    }

    /// Unload all chunks in a column
    pub fn unload_col(&self, col: ColPos) {
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {
                x: col.x,
                y,
                z: col.z,
                realm: col.realm,
            };
            self.chunks.remove(&chunk_pos);
        }
    }

    /// Mark a chunk as changed (triggers mesh rebuild)
    pub fn mark_chunk_changed(&self, chunk_pos: ChunkPos) {
        let _ = self.chunk_changes.send(chunk_pos);
    }
}

/// Event sent when requesting to set a block.
/// This will be picked up by the network system and sent to the server.
#[derive(Message, Debug, Clone)]
pub struct SetBlockRequest {
    pub pos: BlockPos,
    pub block: Block,
}

/// Event sent when the server confirms a block change.
/// Systems should listen to this to update local state.
#[derive(Message, Debug, Clone)]
pub struct BlockChanged {
    pub pos: BlockPos,
    pub old_block: Block,
    pub new_block: Block,
}

/// Event sent when a column is unloaded (either locally or from server)
#[derive(Message, Debug, Clone)]
pub struct ColUnloadEvent(pub ColPos);

/// Plugin to set up the client world system
pub struct ClientWorldPlugin;

impl Plugin for ClientWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SetBlockRequest>()
            .add_event::<BlockChanged>()
            .add_systems(Update, process_block_requests);
    }
}

/// Process block requests: apply locally and send to server.
/// 
/// This uses client-side prediction - we apply the change immediately locally
/// for responsiveness, and also send it to the server. If the server rejects it,
/// the next WorldUpdate will correct our local state.
fn process_block_requests(
    world_map: Option<Res<ClientWorldMap>>,
    mut requests: MessageReader<SetBlockRequest>,
    mut block_changed: MessageWriter<BlockChanged>,
    mut client: Option<ResMut<bevy_renet::renet::RenetClient>>,
) {
    let Some(world_map) = world_map else {
        return;
    };
    
    for request in requests.read() {
        let old_block = world_map.get_block(request.pos);
        
        // Apply the change locally immediately (client-side prediction)
        let (chunk_pos, chunked_pos) = <(ChunkPos, _)>::from(request.pos);
        if let Some(chunk) = world_map.chunks.get(&chunk_pos) {
            chunk.value().write().set(chunked_pos, request.block);
            world_map.mark_chunk_changed(chunk_pos);
            
            block_changed.write(BlockChanged {
                pos: request.pos,
                old_block,
                new_block: request.block,
            });
        }
        
        // Send to server for authoritative processing
        if let Some(ref mut client) = client {
            use crate::network::SendGameMessageExtension;
            use shared::messages::{BlockInteraction, ClientToServerMessage};
            
            client.send_game_message(ClientToServerMessage::BlockInteraction(
                BlockInteraction {
                    pos: request.pos,
                    new_block: request.block,
                },
            ));
        }
    }
}
