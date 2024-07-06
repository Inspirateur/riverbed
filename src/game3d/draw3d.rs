use std::collections::HashMap;
use std::sync::Arc;
use std::thread::yield_now;
use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::view::NoFrustumCulling;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver};
use itertools::{iproduct, Itertools};
use strum::IntoEnumIterator;
use crate::blocks::pos2d::chunks_in_col;
use crate::blocks::{Blocks, ChunkPos, Face, CHUNK_S1, Y_CHUNKS};
use crate::gen::{range_around, ColUnloadEvent, LoadArea, LoadAreaAssigned};
use super::chunk_culling::chunk_culling;
use super::shared_load_area::{setup_shared_load_area, update_shared_load_area, SharedLoadArea};
use super::texture_array::{BlockTextureArray, TexState};
use super::{render3d::Meshable, texture_array::{TextureMap, TextureArrayPlugin}};
const GRID_GIZMO_LEN: i32 = 4;

#[derive(Debug, Component)]
pub struct LOD(pub usize);

fn choose_lod_level(chunk_dist: u32) -> usize {
    if chunk_dist < 8 {
        return 1;
    }
    if chunk_dist < 16 {
        return 2;
    }
    if chunk_dist < 32 {
        return 4;
    }
    if chunk_dist < 64 {
        return 8;
    }
    return 16;
}


fn mark_lod_remesh(
    load_area: Res<LoadArea>, 
    chunk_ents: ResMut<ChunkEntities>, 
    lods: Query<&LOD>, 
    blocks: ResMut<Blocks>
) {
    // FIXME: this only remesh chunks that previously had a mesh 
    // However in some rare cases a chunk with some blocs can produce an empty mesh at certain LODs 
    // and never get remeshed even though it should
    if !load_area.is_changed() { return; }
    for ((chunk_pos, _), entity) in chunk_ents.0.iter().unique_by(|((chunk_pos, _), _)| chunk_pos) {
        let Some(dist) =  load_area.col_dists.get(&(*chunk_pos).into()) else {
            continue;
        };
        let new_lod = choose_lod_level(*dist);
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

fn chunk_aabb_gizmos(mut gizmos: Gizmos, load_area: Res<LoadArea>) {
    for (x, y) in iproduct!(range_around(load_area.center.x, GRID_GIZMO_LEN), 0..=Y_CHUNKS) {
        let start = Vec3::new(x as f32, y as f32, (load_area.center.z-GRID_GIZMO_LEN) as f32)*CHUNK_S1 as f32;
        let end = Vec3::new(x as f32, y as f32, (load_area.center.z+GRID_GIZMO_LEN) as f32)*CHUNK_S1 as f32;
        gizmos.line(start, end, Color::Srgba(css::YELLOW));
    }
    for (z, y) in iproduct!(range_around(load_area.center.z, GRID_GIZMO_LEN), 0..=Y_CHUNKS) {
        let start = Vec3::new((load_area.center.x-GRID_GIZMO_LEN) as f32, y as f32, z as f32)*CHUNK_S1 as f32;
        let end = Vec3::new((load_area.center.x+GRID_GIZMO_LEN) as f32, y as f32, z as f32)*CHUNK_S1 as f32;
        gizmos.line(start, end, Color::Srgba(css::YELLOW));
    }
    for (x, z) in iproduct!(range_around(load_area.center.x, GRID_GIZMO_LEN), range_around(load_area.center.z, GRID_GIZMO_LEN)) {
        let start = Vec3::new(x as f32, 0., z as f32)*CHUNK_S1 as f32;
        let end = Vec3::new(x as f32, Y_CHUNKS as f32, z as f32)*CHUNK_S1 as f32;
        gizmos.line(start, end, Color::Srgba(css::YELLOW));
    }
}

#[derive(Resource)]
pub struct MeshReciever(Receiver<(Option<Mesh>, ChunkPos, Face, LOD)>);

fn setup_mesh_thread(mut commands: Commands, blocks: Res<Blocks>, shared_load_area: Res<SharedLoadArea>, texture_map: Res<TextureMap>) {
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
                let face_meshes = chunks.create_face_meshes(chunk_pos, &texture_map, lod);
                for (i, face_mesh) in face_meshes.into_iter().enumerate() {
                    let face = i.into();
                    if mesh_sender.send((face_mesh, chunk_pos, face, LOD(lod))).is_err() {
                        println!("mesh for {:?} couldn't be sent", chunk_pos)
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
    mut mesh_query: Query<(&mut Handle<Mesh>, &mut LOD)>,
    mut meshes: ResMut<Assets<Mesh>>,
    block_tex_array: Res<BlockTextureArray>,
    load_area: Res<LoadArea>,
    blocks: Res<Blocks>
) {
    let received_meshes: Vec<_> = mesh_reciever.0.try_iter()
        .filter(|(_, chunk_pos, _, _)| load_area.col_dists.contains_key(&(*chunk_pos).into()))
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
                *handle = meshes.add(mesh);
                *old_lod = lod;
            } else {
                // the entity is not instanciated yet, we put it back
                println!("entity wasn't ready to recieve updated mesh");
            }
        } else if blocks.chunks.contains_key(&chunk_pos) {
            let ent = commands.spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(mesh),
                    material: block_tex_array.0.clone_weak(),
                    transform: Transform::from_translation(
                        Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32) * CHUNK_S1 as f32,
                    ),
                    ..Default::default()
                },
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
    mesh_query: Query<&Handle<Mesh>>,
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

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .add_systems(Startup, 
                (setup_shared_load_area, apply_deferred, setup_mesh_thread, apply_deferred)
                .chain()
                .after(LoadAreaAssigned))
            .add_systems(Update, update_shared_load_area)
            .add_systems(Update, mark_lod_remesh)
            .add_systems(Update, pull_meshes.run_if(in_state(TexState::Finished)))
            .add_systems(Update, on_col_unload)
            //.add_systems(Update, chunk_aabb_gizmos)
            .add_systems(PostUpdate, chunk_culling)
            ;
    }
}
