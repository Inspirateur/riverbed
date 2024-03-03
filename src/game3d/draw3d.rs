use std::collections::HashMap;
use std::sync::Arc;
use std::thread::yield_now;
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver};
use itertools::{iproduct, Itertools};
use crate::blocs::{Blocs, ChunkPos, CHUNK_S1, Y_CHUNKS};
use crate::gen::{range_around, ColUnloadEvent, LoadArea, LoadAreaAssigned};
use super::shared_load_area::{setup_shared_load_area, update_shared_load_area, SharedLoadArea};
use super::texture_array::{BlocTextureArray, TexState};
use super::{render3d::Meshable, texture_array::{TextureMap, TextureArrayPlugin}};
const CHUNK_S1_HF: f32 = (CHUNK_S1/2) as f32;
const GRID_GIZMO_LEN: i32 = 4;

#[derive(Debug, Component)]
pub struct LOD(pub usize);

fn choose_lod_level(chunk_dist: u32) -> usize {
    return 1;
    if chunk_dist < 8 {
        return 1;
    }
    if chunk_dist < 16 {
        return 2;
    }
    if chunk_dist < 32 {
        return 4;
    }
    return 8;
}

fn chunk_aabb_gizmos(mut gizmos: Gizmos, load_area: Res<LoadArea>) {
    for (x, y) in iproduct!(range_around(load_area.center.x, GRID_GIZMO_LEN), 0..=Y_CHUNKS) {
        let start = Vec3::new(x as f32, y as f32, (load_area.center.z-GRID_GIZMO_LEN) as f32)*CHUNK_S1 as f32;
        let end = Vec3::new(x as f32, y as f32, (load_area.center.z+GRID_GIZMO_LEN) as f32)*CHUNK_S1 as f32;
        gizmos.line(start, end, Color::YELLOW);
    }
    for (z, y) in iproduct!(range_around(load_area.center.z, GRID_GIZMO_LEN), 0..=Y_CHUNKS) {
        let start = Vec3::new((load_area.center.x-GRID_GIZMO_LEN) as f32, y as f32, z as f32)*CHUNK_S1 as f32;
        let end = Vec3::new((load_area.center.x+GRID_GIZMO_LEN) as f32, y as f32, z as f32)*CHUNK_S1 as f32;
        gizmos.line(start, end, Color::YELLOW);
    }
    for (x, z) in iproduct!(range_around(load_area.center.x, GRID_GIZMO_LEN), range_around(load_area.center.z, GRID_GIZMO_LEN)) {
        let start = Vec3::new(x as f32, 0., z as f32)*CHUNK_S1 as f32;
        let end = Vec3::new(x as f32, Y_CHUNKS as f32, z as f32)*CHUNK_S1 as f32;
        gizmos.line(start, end, Color::YELLOW);
    }
}

#[derive(Resource)]
pub struct MeshReciever(Receiver<(Mesh, ChunkPos, LOD)>);

fn setup_mesh_thread(mut commands: Commands, blocs: Res<Blocs>, shared_load_area: Res<SharedLoadArea>, texture_map: Res<TextureMap>) {
let thread_pool = AsyncComputeTaskPool::get();
    let chunks = Arc::clone(&blocs.chunks);
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
                let mesh = chunks.create_mesh(chunk_pos, &texture_map, lod);
                if mesh_sender.send((mesh, chunk_pos, LOD(lod))).is_err() {
                    println!("mesh for {:?} couldn't be sent", chunk_pos)
                };
            }
        }
    ).detach();
}

pub fn pull_meshes(
    mut commands: Commands, 
    mesh_reciever: Res<MeshReciever>, 
    mut chunk_ents: ResMut<ChunkEntities>, 
    mut mesh_query: Query<(&Handle<Mesh>, &mut LOD, &mut Transform, &mut Aabb)>,
    mut meshes: ResMut<Assets<Mesh>>,
    bloc_tex_array: Res<BlocTextureArray>,
    blocs: Res<Blocs>
) {
    let received_meshes: Vec<_> = mesh_reciever.0.try_iter().collect();
    for (mesh, chunk_pos, lod) in received_meshes.into_iter().rev().unique_by(|(_, pos, _)| *pos) {
        let unit = Vec3::ONE*lod.0 as f32;
        let chunk_s1_hf = CHUNK_S1_HF/lod.0 as f32;
        let chunk_aabb = Aabb {
            center: Vec3A::new(chunk_s1_hf, chunk_s1_hf, chunk_s1_hf),
            half_extents: Vec3A::new(chunk_s1_hf, chunk_s1_hf, chunk_s1_hf)
        };
        if let Some(ent) = chunk_ents.0.get(&chunk_pos) {
            if let Ok((handle, mut old_lod, mut transform, mut aabb)) = mesh_query.get_mut(*ent) {
                if let Some(old_mesh) = meshes.get_mut(handle) {
                    *old_mesh = mesh;
                    transform.scale = unit;
                    *old_lod = lod;
                    *aabb = chunk_aabb;
                }
            } else {
                // the entity is not instanciated yet, we put it back
                println!("entity wasn't ready to recieve updated mesh");
            }
        } else if blocs.chunks.contains_key(&chunk_pos) {
            let ent = commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: bloc_tex_array.0.clone(),
                transform: Transform::from_translation(
                    Vec3::new(chunk_pos.x as f32, chunk_pos.y as f32, chunk_pos.z as f32) * CHUNK_S1 as f32,
                ),
                ..Default::default()
            }).insert(chunk_aabb).insert(lod).id();
            chunk_ents.0.insert(chunk_pos, ent);
        }
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut chunk_ents: ResMut<ChunkEntities>,
) {
    for col_ev in ev_unload.read() {
        for i in 0..Y_CHUNKS {
            if let Some(ent) = chunk_ents.0.remove(&ChunkPos {
                x: col_ev.0.x,
                y: i as i32,
                z: col_ev.0.z,
                realm: col_ev.0.realm
            }) {
                commands.entity(ent).despawn();
            }
        }
    }
}


#[derive(Resource)]
pub struct ChunkEntities(pub HashMap::<ChunkPos, Entity>);

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
            .add_systems(Update, pull_meshes.run_if(in_state(TexState::Finished)))
            .add_systems(Update, on_col_unload)
            // .add_systems(Update, chunk_aabb_gizmos)
            ;
    }
}
