use std::collections::HashMap;
use std::convert::identity;
use std::iter::zip;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{ShaderRef, AsBindGroup};
use leafwing_input_manager::prelude::ActionState;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};
use ourcraft::{Pos, Blocs, ChunkPos2D, Bloc, ChunkPos, CHUNK_S1};
use crate::render2d::Render2D;
use crate::render3d::Meshable;
use crate::texture_array::{TextureMap, TextureArrayPlugin};
use crate::{player::Dir, load_cols::{ColLoadEvent, ColUnloadEvent, ColEntities}};


pub fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 50., 0.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}


pub fn update_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Pos, (With<ActionState<Dir>>, Changed<Pos>)>
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_pos) = player_query.get_single() {
            cam_pos.translation.x = player_pos.x;
            cam_pos.translation.y = player_pos.y;
            cam_pos.translation.z = player_pos.z;
        }
    }
}

pub fn on_col_load(
    mut commands: Commands,
    mut ev_load: EventReader<ColLoadEvent>,
    blocs: Res<Blocs>,
    mut col_ents: ResMut<ColEntities>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_map: Res<TextureMap>
) {
    let cols: Vec<_> = ev_load.iter().map(|col_ev| col_ev.0).collect();
    let mut ents = Vec::new();
    for col in cols.iter().filter(|col| blocs.0.contains_key(*col)) {
        for (cy, chunk) in blocs.0.get(col).unwrap().chunks.iter().enumerate().rev() {
            if chunk.is_some() {
                let ent = commands.spawn(PbrBundle {
                    mesh: meshes.add(blocs.fast_mesh(ChunkPos {x: col.x, y: cy as i32, z: col.z, realm: col.realm}, &texture_map)),
                    material: materials.add(Color::rgb(0.7, 0.3, 0.7).into()),
                    transform: Transform::from_translation(
                        Vec3::new(col.x as f32, cy as f32, col.z as f32) * (CHUNK_S1/2) as f32,
                    ),
                    ..Default::default()
                }).id();
                ents.push(ent);
            }
        }
        println!("Loaded ({:?})", col);
    }
    for (col, ent) in zip(&cols, &ents) {
        col_ents.0.insert(*col, *ent);
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut col_ents: ResMut<ColEntities>,
) {
    for col_ev in ev_unload.iter() {
        if let Some(ent) = col_ents.0.remove(&col_ev.0) {
            commands.entity(ent).despawn();
        }
    }
}


pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ColEntities(HashMap::<ChunkPos2D, Entity>::new()))
            .add_plugins(TextureArrayPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update_cam)
            .add_systems(Update, on_col_load)
            .add_systems(Update, on_col_unload);
    }
}
