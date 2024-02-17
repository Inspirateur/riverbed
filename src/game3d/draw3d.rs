use std::collections::HashMap;
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use crate::agents::PlayerControlled;
use crate::blocs::{Blocs, ChunkPos, CHUNK_S1, Y_CHUNKS};
use crate::gen::{ColUnloadEvent, LoadArea};
use super::texture_array::{BlocTextureArray, TexState};
use super::{render3d::Meshable, texture_array::{TextureMap, TextureArrayPlugin}};
const CHUNK_S1_HF: f32 = (CHUNK_S1/2) as f32;


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
    return 8;
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

pub fn process_bloc_changes(
    mut commands: Commands,
    mesh_query: Query<&Handle<Mesh>>,
    load_area_query: Query<&LoadArea, With<PlayerControlled>>,
    mut blocs: ResMut<Blocs>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_ents: ResMut<ChunkEntities>,
    texture_map: Res<TextureMap>,
    bloc_tex_array: Res<BlocTextureArray>,
) {
    let Ok(load_area) = load_area_query.get_single() else {
        return;
    };

    if let Some(chunk) = blocs.changes.pop_front() {
        let Some(col_dist) = load_area.col_dists.get(&chunk.into()) else { return; };
        let lod = choose_lod_level(*col_dist);
        if let Some(ent) = chunk_ents.0.get(&chunk) {
            if let Ok(handle) = mesh_query.get_component::<Handle<Mesh>>(*ent) {
                if let Some(mesh) = meshes.get_mut(handle) {
                    blocs.update_mesh(chunk, mesh, &texture_map, lod);
                }
            } else {
                // the entity is not instanciated yet, we put it back
                blocs.changes.push_back(chunk);
            }
        } else {
            let chunk_s1_hf = CHUNK_S1_HF/lod as f32;
            let chunk_aabb = Aabb {
                center: Vec3A::new(chunk_s1_hf, chunk_s1_hf, chunk_s1_hf),
                half_extents: Vec3A::new(chunk_s1_hf, chunk_s1_hf, chunk_s1_hf)
            };
            let unit = Vec3::ONE*lod as f32;
            let ent = commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(blocs.create_mesh(chunk, &texture_map, lod)),
                material: bloc_tex_array.0.clone(),
                transform: Transform::from_translation(
                    Vec3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32) * CHUNK_S1 as f32 - unit,
                ).with_scale(unit),
                ..Default::default()
            }).insert(chunk_aabb).insert(LOD(lod)).id();
            chunk_ents.0.insert(chunk, ent);
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
            .add_systems(Update, on_col_unload)
            // TODO: need to thread this so it can run as fast as possible but in the meantime running it twice is decent
            .add_systems(Update, process_bloc_changes.run_if(in_state(TexState::Finished)))
            ;
    }
}
