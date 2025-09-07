use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::view::NoFrustumCulling;
use itertools::Itertools;
use strum::IntoEnumIterator;
use crate::agents::PlayerControlled;
use crate::block::Face;
use crate::render::mesh_thread::{setup_mesh_thread, update_shared_load_area, MeshReciever, SharedPlayerCol};
use crate::render::MeshOrderSender;
use crate::world::pos2d::chunks_in_col;
use crate::world::{ChunkPos, ColUnloadEvent, PlayerCol, VoxelWorld, CHUNK_S1};
use super::chunk_culling::chunk_culling;
use super::texture_array::BlockTextureArray;
use super::BlockTexState;
use super::texture_array::TextureArrayPlugin;

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .insert_resource(SharedPlayerCol::default())
            .add_systems(OnEnter(BlockTexState::Mapped),  setup_mesh_thread)
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

pub fn choose_lod_level(chunk_dist: u32) -> usize {
    if chunk_dist < 16 {
        return 1;
    }
    return 2;
}

fn mark_lod_remesh(
    player_query: Single<&PlayerCol, (With<PlayerControlled>, Changed<PlayerCol>)>, 
    chunk_ents: ResMut<ChunkEntities>, 
    lods: Query<&LOD>, 
    mesh_order_sender: Res<MeshOrderSender>,
) {
    // FIXME: this only remesh chunks that previously had a mesh 
    // However in some rare cases a chunk with some blocks can produce an empty mesh at certain LODs 
    // and never get remeshed even though it should
    let player_col = player_query.0;
    for ((chunk_pos, _), entity) in chunk_ents.0.iter().unique_by(|((chunk_pos, _), _)| chunk_pos) {
        let dist =  player_col.dist((*chunk_pos).into());
        let new_lod = choose_lod_level(dist as u32);
        let Ok(old_lod) = lods.get(*entity) else {
            continue;
        };
        if new_lod != old_lod.0 {
            mesh_order_sender.0.send(*chunk_pos).expect("MeshOrderSender channel is closed");
        }
    }
}

pub fn pull_meshes(
    mut commands: Commands, 
    mesh_reciever: Res<MeshReciever>, 
    mut chunk_ents: ResMut<ChunkEntities>, 
    mut mesh_query: Query<(&mut Mesh3d, &mut LOD)>,
    mut meshes: ResMut<Assets<Mesh>>,
    block_tex_array: Res<BlockTextureArray>,
    blocks: Res<VoxelWorld>
) {
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
        } else if blocks.chunks.read().contains_key(&chunk_pos) {
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