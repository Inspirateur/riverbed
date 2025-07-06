use std::collections::HashMap;
use std::sync::Arc;
use std::thread::yield_now;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::view::NoFrustumCulling;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver};
use itertools::Itertools;
use parking_lot::RwLock;
use strum::IntoEnumIterator;
use crate::agents::PlayerControlled;
use crate::block::Face;
use crate::logging::LogData;
use crate::world::pos2d::chunks_in_col;
use crate::world::{ChunkPos, ColPos, ColUnloadEvent, PlayerCol, VoxelWorld, CHUNK_S1};
use super::chunk_culling::chunk_culling;
use super::texture_array::BlockTextureArray;
use super::BlockTexState;
use super::texture_array::{TextureMap, TextureArrayPlugin};
const GRID_GIZMO_LEN: i32 = 4;

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .insert_resource(SharedPlayerCol::default())
            .add_systems(Startup, setup_mesh_thread)
            .add_systems(Update, update_shared_load_area)
            .add_systems(Update, mark_lod_remesh)
            .add_systems(Update, pull_meshes.run_if(in_state(BlockTexState::Mapped)))
            .add_systems(Update, on_col_unload)
            .add_systems(PostUpdate, chunk_culling)
            ;
    }
}

#[derive(Debug, Component)]
pub struct LOD(pub usize);

fn choose_lod_level(chunk_dist: u32) -> usize {
    if chunk_dist < 16 {
        return 1;
    }
    return 2;
}

fn mark_lod_remesh(
    player_query: Single<&PlayerCol, (With<PlayerControlled>, Changed<PlayerCol>)>, 
    chunk_ents: ResMut<ChunkEntities>, 
    lods: Query<&LOD>, 
    blocks: ResMut<VoxelWorld>
) {
    // FIXME: this only remesh chunks that previously had a mesh 
    // However in some rare cases a chunk with some blocs can produce an empty mesh at certain LODs 
    // and never get remeshed even though it should
    let player_col = player_query.0;
    for ((chunk_pos, _), entity) in chunk_ents.0.iter().unique_by(|((chunk_pos, _), _)| chunk_pos) {
        let dist =  player_col.dist((*chunk_pos).into());
        let new_lod = choose_lod_level(dist as u32);
        let Ok(old_lod) = lods.get(*entity) else {
            continue;
        };
        if new_lod != old_lod.0 {
            let Some(mut chunk) = blocks.chunks.get_mut(chunk_pos) else {
                continue;
            };
            chunk.changed = true;
        }
    }
}

#[derive(Resource)]
pub struct MeshReciever(Receiver<(Option<Mesh>, ChunkPos, Face, LOD)>);

fn setup_mesh_thread(mut commands: Commands, blocks: Res<VoxelWorld>, shared_load_area: Res<SharedPlayerCol>, texture_map: Res<TextureMap>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let chunks = Arc::clone(&blocks.chunks);
    let (mesh_sender, mesh_reciever) = unbounded();
    commands.insert_resource(MeshReciever(mesh_reciever));
    let shared_load_area = Arc::clone(&shared_load_area.0);
    let texture_map = Arc::clone(&texture_map.0);
    thread_pool.spawn(
        async move {
            while texture_map.len() == 0 {
                yield_now()
            }
            loop {
                let Some((chunk_pos, dist)) = shared_load_area.read().pop_closest_change(&chunks) else {
                    yield_now();
                    continue;
                };
                let lod = choose_lod_level(dist);
                let Some(chunk) = chunks.get(&chunk_pos) else {
                    continue;
                };
                let face_meshes = chunk.create_face_meshes(&*texture_map, lod);
                trace!("{}", LogData::ChunkMeshed(chunk_pos));
                for (i, face_mesh) in face_meshes.into_iter().enumerate() {
                    let face = i.into();
                    if mesh_sender.send((face_mesh, chunk_pos, face, LOD(lod))).is_err() {
                        warn!("mesh for {:?} couldn't be sent", chunk_pos)
                    };
                }
            }
        }
    ).detach();
}

pub fn pull_meshes(
    mut commands: Commands, 
    mesh_reciever: Res<MeshReciever>, 
    mut chunk_ents: ResMut<ChunkEntities>, 
    mut mesh_query: Query<(&mut Mesh3d, &mut LOD)>,
    mut meshes: ResMut<Assets<Mesh>>,
    block_tex_array: Res<BlockTextureArray>,
    player_query: Single<&PlayerCol, With<PlayerControlled>>,
    blocks: Res<VoxelWorld>
) {
    let player_col = player_query.0;
    let received_meshes: Vec<_> = mesh_reciever.0.try_iter()
        .collect();
    for (mesh_opt, chunk_pos, face, lod) in received_meshes
        .into_iter().rev().unique_by(|(_, pos, face, _)| (*pos, *face)) 
    {
        let Some(mesh) = mesh_opt else {
            if let Some(ent) = chunk_ents.0.remove(&(chunk_pos, face)) {
                commands.entity(ent).despawn();
            }
            continue;
        };
        let chunk_aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_S1 as f32));
        if let Some(ent) = chunk_ents.0.get(&(chunk_pos, face)) {
            if let Ok((mut handle, mut old_lod)) = mesh_query.get_mut(*ent) {
                handle.0 = meshes.add(mesh);
                *old_lod = lod;
            } else {
                // the entity is not instanciated yet, we put it back
                println!("entity wasn't ready to recieve updated mesh");
            }
        } else if blocks.chunks.contains_key(&chunk_pos) {
            let ent = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(block_tex_array.0.clone_weak()),
                Transform::from_translation(
                    Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32) * CHUNK_S1 as f32,
                ),
                NoFrustumCulling,
                chunk_aabb, 
                lod, 
                face
            )).id();
            chunk_ents.0.insert((chunk_pos, face), ent);
        }
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut chunk_ents: ResMut<ChunkEntities>,
    mesh_query: Query<&Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for col_ev in ev_unload.read() {
        for chunk_pos in chunks_in_col(&col_ev.0) {
            for face in Face::iter() {
                if let Some(ent) = chunk_ents.0.remove(&(chunk_pos, face)) {
                    if let Ok(handle) = mesh_query.get(ent) {
                        meshes.remove(handle);
                    }
                    commands.entity(ent).despawn();
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct ChunkEntities(pub HashMap::<(ChunkPos, Face), Entity>);

impl ChunkEntities {
    pub fn new() -> Self {
        ChunkEntities(HashMap::new())
    }
}

#[derive(Default, Resource)]
pub struct SharedPlayerCol(pub Arc<RwLock<ColPos>>);

pub fn update_shared_load_area(player_query: Single<&PlayerCol, (With<PlayerControlled>, Changed<PlayerCol>)>, shared_load_area: Res<SharedPlayerCol>) {
    let player_col = player_query.0;
    *shared_load_area.0.write() = player_col.clone();
}