use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver, Sender};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread::yield_now;
use crate::agents::PlayerControlled;
use crate::block::Face;
use crate::logging::LogData;
use crate::render::mesh_draw::{choose_lod_level, LOD};
use crate::render::texture_array::TextureMap;
use crate::world::{ChunkPos, ColPos, PlayerCol, VoxelWorld};

pub fn setup_mesh_thread(mut commands: Commands, blocks: Res<VoxelWorld>, texture_map: Res<TextureMap>, shared_load_area: Res<SharedPlayerCol>, mesh_order_receiver: Res<MeshOrderReceiver>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let chunks = Arc::clone(&blocks.chunks);
    let (mesh_sender, mesh_reciever) = unbounded();
    commands.insert_resource(MeshReciever(mesh_reciever));
    let texture_map = Arc::clone(&texture_map.0);
    let mesh_order_receiver = mesh_order_receiver.0.clone();
    let shared_load_area = shared_load_area.0.clone();
    thread_pool.spawn(
        async move {
            // Busy wait until the texture map is loaded (ugly but only costly on startup)
            while texture_map.len() == 0 {
                yield_now()
            }
            let mut mesh_cache: HashSet<ChunkPos> = HashSet::new();
            let mut mesh_orders: Vec<ChunkPos> = Vec::new();
            'outer: loop {
                loop {
                    // If mesh_orders is empty, we block on mesh order updates to not waste resources
                    let chunk_pos = if mesh_orders.len() == 0 {
                        let Ok(pos) = mesh_order_receiver.recv() else {
                            warn!("MeshOrder channel is closed, stopping mesh thread");
                            break 'outer;
                        };
                        pos
                    } else {
                        match mesh_order_receiver.try_recv() {
                            Ok(update) => update,
                            Err(_) => break, // no more updates, exit the loop
                        }
                    };
                    if mesh_cache.insert(chunk_pos) {
                        mesh_orders.push(chunk_pos);
                    }
                }
                let player_col = shared_load_area.read_arc().clone();
                // Pop the closest mesh order
                let (i, (chunk_pos, dist)) = mesh_orders
                    .iter()
                    .map(|&chunk_pos| {
                        let dist = player_col.dist(chunk_pos.into());
                        (chunk_pos, dist)
                    })
                    .enumerate()
                    .min_by_key(|(_, (_, dist))| *dist)
                    .unwrap();
                mesh_orders.remove(i);
                mesh_cache.remove(&chunk_pos);
                let lod = choose_lod_level(dist as u32);
                let Some(chunk) = chunks.get(&chunk_pos) else {
                    continue;
                };
                let face_meshes = chunk.create_face_meshes(&*texture_map, lod);
                trace!("{}", LogData::ChunkMeshed(chunk_pos));
                for (i, face_mesh) in face_meshes.into_iter().enumerate() {
                    let face = i.into();
                    if mesh_sender.send((face_mesh, chunk_pos, face, LOD(lod))).is_err() {
                        warn!("Mesh channel is closed, stopping mesh thread");
                        break 'outer;
                    };
                }
            }
        }
    ).detach();
}

#[derive(Resource)]
pub struct MeshReciever(pub Receiver<(Option<Mesh>, ChunkPos, Face, LOD)>);

#[derive(Resource)]
pub struct MeshOrderSender(pub Sender<ChunkPos>);

#[derive(Resource)]
pub struct MeshOrderReceiver(pub Receiver<ChunkPos>);

#[derive(Default, Resource, Clone)]
pub struct SharedPlayerCol(pub Arc<RwLock<ColPos>>);

pub fn update_shared_load_area(player_query: Single<&PlayerCol, (With<PlayerControlled>, Changed<PlayerCol>)>, shared_load_area: Res<SharedPlayerCol>) {
    let player_col = player_query.0;
    *shared_load_area.0.write() = player_col.clone();
}