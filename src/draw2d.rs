use ourcraft::{CHUNK_S1, Bloc, Blocs, Pos, ChunkPos2D, HashMapUtils};
use crate::load_cols::{ColLoadOrders, ColUnloadEvent};
use crate::player::Dir;
use anyhow::Result;
use bevy::prelude::*;
use colorsys::Rgb;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashMap;
use std::str::FromStr;
use crate::render2d::Render2D;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..Default::default()
        },
        transform: Transform::from_xyz(0., 50., 10.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

pub fn update_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Pos, (With<ActionState<Dir>>, Changed<Pos>)>,
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_pos) = player_query.get_single() {
            cam_pos.translation.x = player_pos.x;
            cam_pos.translation.z = player_pos.z;
        }
    }
}

pub fn on_col_load(
    mut commands: Commands,
    mut ev_load: ResMut<ColLoadOrders>,
    mut blocs: ResMut<Blocs>,
    soil_color: Res<SoilColor>,
    im_query: Query<&Handle<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut col_ents: ResMut<ColEntities>,
) {
    // Add the rendered column before registering it
    if let Some(col) = ev_load.0.pop_back() {
        println!("Loaded ({:?})", col);
        let trans = Vec3::new(col.x as f32, 0., col.z as f32) * CHUNK_S1 as f32;
        let ent = commands
            .spawn(SpriteBundle {
                texture: images.add(blocs.render_col(col, &soil_color)),
                transform: Transform::from_translation(trans)
                    .looking_at(trans + Vec3::Y, Vec3::Y),
                ..default()
            })
            .id();
        col_ents.0.insert(col, ent);
        // if there was an already loaded col below
        let col_below = col + <Vec3>::from(Dir::Back);
        if let Some(ent_below) = col_ents.0.get(&col_below) {
            if let Ok(handle) = im_query.get_component::<Handle<Image>>(*ent_below) {
                if let Some(image) = images.get_mut(&handle) {
                    // update the top side shading with the new information
                    blocs.update_side(image, col_below, &soil_color);
                }
            }
        }
    }
}

pub fn on_col_unload(
    mut blocs: ResMut<Blocs>,
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut col_ents: ResMut<ColEntities>,
) {
    for col_ev in ev_unload.iter() {
        if let Some(ent) = col_ents.0.remove(&col_ev.0) {
            commands.entity(ent).despawn();
        } else {
            println!("unload order for {:?} but no entities", col_ev.0);
        }
    }
}

pub fn process_bloc_changes(
    mut blocs: ResMut<Blocs>, 
    im_query: Query<&Handle<Image>>,
    mut images: ResMut<Assets<Image>>,
    col_ents: Res<ColEntities>,
    soil_color: Res<SoilColor>,
) {
    if let Some((chunk, changes)) = blocs.changes.pop() {
        let col = chunk.into();
        if let Some(ent) = col_ents.0.get(&col) {
            if let Ok(handle) = im_query.get_component::<Handle<Image>>(*ent) {
                if let Some(image) = images.get_mut(&handle) {
                    blocs.process_changes(chunk, changes, image, &soil_color);
                }
            } else {
                blocs.changes.insert(chunk, changes);
            }
        }
    }
}

#[derive(Resource)]
pub struct SoilColor(pub HashMap<Bloc, Rgb>);

impl SoilColor {
    pub fn from_csv(path: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut data = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let color = Rgb::from_hex_str(&record[1].trim())?;
            data.insert(Bloc::from_str(&record[0]).unwrap(), color);
        }
        Ok(SoilColor(data))
    }
}

#[derive(Resource)]
pub struct ColEntities(pub HashMap::<ChunkPos2D, Entity>);

impl ColEntities {
    pub fn new() -> Self {
        ColEntities(HashMap::new())
    }
}

pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoilColor::from_csv("assets/data/soils_color.csv").unwrap())
            .insert_resource(ColEntities::new())
            .add_systems(Startup, setup)
            .add_systems(Update, update_cam)
            .add_systems(Update, on_col_load)
            .add_systems(Update, on_col_unload)
            .add_systems(Update, process_bloc_changes)
            ;
    }
}
