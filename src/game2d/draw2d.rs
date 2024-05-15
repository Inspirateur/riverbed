use crate::blocks::{Block, ColPos};
use crate::gen::ColUnloadEvent;
use crate::agents::{PlayerControlled, AABB};
use anyhow::Result;
use bevy::prelude::*;
use colorsys::Rgb;
use std::collections::HashMap;

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CameraSpawn;

pub fn setup(mut commands: Commands, player_query: Query<(Entity, &AABB), With<PlayerControlled>>) {
    let (player, aabb) = player_query.get_single().unwrap();
    let cam = commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..Default::default()
        },
        transform: Transform::from_xyz(aabb.0.x/2., 2., aabb.0.z/2.)
            .looking_at(Vec3::new(aabb.0.x/2., 0., aabb.0.z/2.), Vec3::Y),
        ..Default::default()
    }).id();
    commands.entity(player).add_child(cam);
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut col_ents: ResMut<ColEntities>,
) {
    for col_ev in ev_unload.read() {
        if let Some(ent) = col_ents.0.remove(&col_ev.0) {
            commands.entity(ent).despawn();
        }
    }
}
/* 
pub fn process_chunk_changes(
    mut commands: Commands,
    load_area_query: Query<&LoadArea, With<PlayerControlled>>,
    im_query: Query<&Handle<Image>>,
    blocks: ResMut<Blocks>, 
    mut images: ResMut<Assets<Image>>,
    mut col_ents: ResMut<ColEntities>,
    soil_color: Res<SoilColor>,
) {
    let Ok(load_area) = load_area_query.get_single() else {
        return;
    };

    if let Some(chunk) = blocks.changes.pop() {
        let col: ColPos = chunk.into();
        if !load_area.col_dists.contains_key(&col) { return; }
        if let Some(ent) = col_ents.0.get(&col) {
            if let Ok(handle) = im_query.get_component::<Handle<Image>>(*ent) {
                if let Some(image) = images.get_mut(handle) {
                    blocks.update_image(chunk.into(), image, &soil_color);
                }
            } else {
                // the entity is not instanciated yet, we put it back
                blocks.changes.push(chunk);
            }
        } else {
            let trans = Vec3::new(col.x as f32, 0., col.z as f32) * CHUNK_S1 as f32;
            let ent = commands
                .spawn(SpriteBundle {
                    texture: images.add(blocks.create_image(col, &soil_color)),
                    transform: Transform::from_translation(trans)
                        .looking_at(trans + Vec3::Y, Vec3::Y),
                    ..default()
                })
                .id();
            col_ents.0.insert(col, ent);
        }
    }
}*/

#[derive(Resource)]
pub struct SoilColor(pub HashMap<Block, Rgb>);

impl SoilColor {
    pub fn from_csv(path: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut data = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let color = Rgb::from_hex_str(&record[1].trim())?;
            if let Ok(block) = ron::from_str(&record[0]) {
                data.insert(block, color);
            } else {
                warn!(target: "ourcraft", "Block '{}' from soil_color.csv doesn't exist", &record[0]);
            }
        }
        Ok(SoilColor(data))
    }
}

#[derive(Resource)]
pub struct ColEntities(pub HashMap::<ColPos, Entity>);

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
            .add_systems(Update, on_col_unload)
            //.add_systems(Update, process_chunk_changes)
            ;
    }
}
