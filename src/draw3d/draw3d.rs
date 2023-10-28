use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use leafwing_input_manager::prelude::*;
use crate::blocs::{Blocs, ChunkPos, CHUNK_S1, Y_CHUNKS, ChunkChanges};
use crate::GameState;
use crate::gen::{LoadedCols, ColUnloadEvent};
use crate::sky::SkyPlugin;
use super::{render3d::Meshable, texture_array::{TextureMap, TextureArrayPlugin}, camera::*};

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut chunk_ents: ResMut<ChunkEntities>,
) {
    for col_ev in ev_unload.iter() {
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
    loaded_cols: Res<LoadedCols>,
    mut commands: Commands,
    mut blocs: ResMut<Blocs>, 
    mesh_query: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_ents: ResMut<ChunkEntities>,
    texture_map: Res<TextureMap>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    if let Some((chunk, chunk_change)) = blocs.changes.pop() {
        if !loaded_cols.has_player(chunk.into()) { return; }
        match chunk_change {
            ChunkChanges::Created => {
                let ent = commands.spawn(PbrBundle {
                    mesh: meshes.add(blocs.fast_mesh(chunk, &texture_map)),
                    material: materials.add(Color::rgb(
                        if chunk.x % 2 == 0 { 0.8 } else { 0.4 }, 
                        if chunk.y % 2 == 0 { 0.8 } else { 0.4 }, 
                        if chunk.z % 2 == 0 { 0.8 } else { 0.4 }
                    ).into()),
                    transform: Transform::from_translation(
                        Vec3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32) * CHUNK_S1 as f32 - Vec3::new(1., 1., 1.),
                    ),
                    ..Default::default()
                }).insert(NoFrustumCulling).id();
                // this should not happen
                assert!(!chunk_ents.0.contains_key(&chunk));
                chunk_ents.0.insert(chunk, ent);
            },
            ChunkChanges::Edited(changes) => {
                if let Some(ent) = chunk_ents.0.get(&chunk) {
                    if let Ok(handle) = mesh_query.get_component::<Handle<Mesh>>(*ent) {
                        if let Some(mesh) = meshes.get_mut(&handle) {
                            blocs.process_changes(chunk, changes, mesh, &texture_map);
                        }
                    } else {
                        // the entity is not instanciated yet, we put it back
                        blocs.changes.insert(chunk, ChunkChanges::Edited(changes));
                    }
                }
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
            .add_plugins(SkyPlugin)
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .add_systems(Startup, setup)
            .add_systems(Update, translate_cam)
            .add_systems(Update, pan_camera.run_if(in_state(GameState::Game)))
            .add_systems(Update, target_bloc)
            .add_systems(Update, apply_fps_cam)
            .add_systems(Update, on_col_unload)
            .add_systems(Update, process_bloc_changes)
            .add_systems(Update, bloc_outline)
            ;
    }
}
