use crate::Block;
use crate::block::{Face, FaceSpecifier};
use crate::world::{Chunk, GridChunkPos};
use avian3d::prelude::Collider;
use bevy::prelude::Mesh;
use crossbeam::channel::{Receiver, unbounded};
use crossbeam_skiplist::SkipMap;
use hashbrown::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;

pub type GridMeshOutput = (Option<Mesh>, GridChunkPos, Face);
pub type GridColliderOutput = (Option<Collider>, GridChunkPos);

/// Spawns a detached worker that meshes chunks of one voxel grid. The worker
/// drains `orders` and produces (per chunk) six face meshes plus one trimesh
/// collider, sending them on the returned receivers.
///
/// The texture map is cloned into the worker. Callers must therefore only
/// invoke this once `BlockTexState::Mapped` has been entered, otherwise the
/// worker captures an empty map and emits untextured meshes forever.
pub fn spawn_grid_mesh_thread(
    chunks: Arc<SkipMap<GridChunkPos, RwLock<Chunk>>>,
    texture_map: HashMap<(Block, FaceSpecifier), usize>,
    orders: Receiver<GridChunkPos>,
) -> (Receiver<GridMeshOutput>, Receiver<GridColliderOutput>) {
    let (mesh_sender, mesh_receiver) = unbounded::<GridMeshOutput>();
    let (collider_sender, collider_receiver) = unbounded::<GridColliderOutput>();
    // Use std::thread::spawn rather than AsyncComputeTaskPool: each grid
    // gets its own OS thread that blocks on `orders.recv()` without
    // competing with other Bevy/Avian work for the shared task-pool threads.
    thread::spawn(move || {
        while let Ok(chunk_pos) = orders.recv() {
            let Some(entry) = chunks.get(&chunk_pos) else {
                continue;
            };
            let chunk_guard = entry.value().read();
            let face_meshes = chunk_guard.create_face_meshes(&texture_map, 1, None);
            let collider = chunk_guard
                .create_collider_data()
                .map(|(verts, idx)| Collider::trimesh(verts, idx));
            drop(chunk_guard);
            if collider_sender.send((collider, chunk_pos)).is_err() {
                return;
            }
            for (face_n, mesh) in face_meshes.into_iter().enumerate() {
                if mesh_sender.send((mesh, chunk_pos, face_n.into())).is_err() {
                    return;
                }
            }
        }
    });
    (mesh_receiver, collider_receiver)
}
