use crate::Block;
use crate::world::{CHUNK_S1, Chunk, ChunkedPos, chunked};
use bevy::prelude::*;
use crossbeam::channel::Sender;
use crossbeam_skiplist::SkipMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// Block coordinate inside a voxel grid's local frame. Mirrors `BlockPos` but
/// drops `Realm` — each grid is its own coordinate system, anchored to the
/// owning entity's `Transform.translation`.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct GridBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Chunk coordinate inside a voxel grid. Stride is `CHUNK_S1`, same as the
/// world's `ChunkPos`.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct GridChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<GridBlockPos> for (GridChunkPos, ChunkedPos) {
    fn from(p: GridBlockPos) -> Self {
        let (cx, dx) = chunked::<CHUNK_S1, 1>(p.x);
        let (cy, dy) = chunked::<CHUNK_S1, 1>(p.y);
        let (cz, dz) = chunked::<CHUNK_S1, 1>(p.z);
        (
            GridChunkPos {
                x: cx,
                y: cy,
                z: cz,
            },
            ChunkedPos {
                x: dx,
                y: dy,
                z: dz,
            },
        )
    }
}

impl From<GridChunkPos> for Vec3 {
    fn from(p: GridChunkPos) -> Self {
        Vec3::new(p.x as f32, p.y as f32, p.z as f32) * CHUNK_S1 as f32
    }
}

/// Voxel data for a movable rigidbody grid. Lives as a `Component` on the
/// grid's root entity. Storage layout mirrors `VoxelWorld`: a thread-safe
/// skiplist of chunks behind an `Arc`, plus a sender that signals which chunks
/// need re-meshing.
#[derive(Component, Clone)]
pub struct VoxelGrid {
    pub chunks: Arc<SkipMap<GridChunkPos, RwLock<Chunk>>>,
    pub(crate) chunk_changes: Sender<GridChunkPos>,
}

impl VoxelGrid {
    pub fn new(chunk_changes: Sender<GridChunkPos>) -> Self {
        Self {
            chunks: Arc::new(SkipMap::new()),
            chunk_changes,
        }
    }

    pub fn set_block(&self, pos: GridBlockPos, block: Block) {
        let (chunk_pos, chunked_pos) = <(GridChunkPos, ChunkedPos)>::from(pos);
        self.chunks
            .get_or_insert_with(chunk_pos, || RwLock::new(Chunk::new()))
            .value()
            .write()
            .set(chunked_pos, block);
        // The receiver lives on the per-grid mesh worker; if it's been dropped
        // (grid despawned mid-build), the order is silently discarded.
        let _ = self.chunk_changes.send(chunk_pos);
    }

    pub fn get_block(&self, pos: GridBlockPos) -> Block {
        let (chunk_pos, chunked_pos) = <(GridChunkPos, ChunkedPos)>::from(pos);
        match self.chunks.get(&chunk_pos) {
            Some(entry) => entry.value().read().get(chunked_pos).clone(),
            None => Block::Air,
        }
    }
}
