use crate::bloc::Bloc;
use crate::chunk::{Chunk, CHUNK_S1, CHUNK_S2};
use crate::load_cols::{ColLoadEvent, ColUnloadEvent};
use crate::player::Action;
use crate::pos::Pos;
use crate::realm::Realm;
use crate::world_data::{WorldData, WATER_H};
use anyhow::Result;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::texture::BevyDefault;
use colorsys::Rgb;
use itertools::iproduct;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashMap;
use std::str::FromStr;

pub fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn update_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Pos, (With<ActionState<Action>>, Changed<Pos>)>,
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_pos) = player_query.get_single() {
            cam_pos.translation = Vec3::new(player_pos.coord.x, player_pos.coord.z, 0.);
        }
    }
}

fn top_bloc(col: &[Option<Chunk>], i: usize) -> (Bloc, i32) {
    let (dx, dz) = (i % CHUNK_S1, i / CHUNK_S1);
    for cy in (0..col.len()).rev() {
        if let Some(chunk) = &col[cy] {
            for dy in (0..CHUNK_S1).rev() {
                let bloc = chunk.get(dx, dy, dz);
                if *bloc != Bloc::Air {
                    return (*bloc, (cy * CHUNK_S1 + dy) as i32);
                }
            }
        }
    }
    (Bloc::Bedrock, 0)
}

pub fn render(realm: Realm, x: i32, z: i32, world: &WorldData, soil_color: &SoilColor) -> Image {
    let mut data = vec![255; CHUNK_S2];
    let col = world.chunks.col(realm, x, z);
    let def_color = Rgb::default();
    let water_color = Rgb::new(0.1, 0.5, 0.9, None);
    for i in (0..CHUNK_S2 * 4).step_by(4) {
        let (bloc, y) = top_bloc(col, i / 4);
        let color = if y > WATER_H {
            soil_color.0.get(&bloc).unwrap_or(&def_color)
        } else {
            &water_color
        };
        data[i] = (color.red() * 255.) as u8;
        data[i + 1] = (color.green() * 255.) as u8;
        data[i + 2] = (color.blue() * 255.) as u8;
    }
    let img = Image::new(
        Extent3d {
            width: CHUNK_S1 as u32,
            height: CHUNK_S1 as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        BevyDefault::bevy_default(),
    );
    img
}

pub fn on_col_load(
    mut commands: Commands,
    mut ev_load: EventReader<ColLoadEvent>,
    world: Res<WorldData>,
    soil_color: Res<SoilColor>,
    mut images: ResMut<Assets<Image>>,
    mut col_ents: ResMut<HashMap<(Realm, i32, i32), Entity>>,
) {
    for ColLoadEvent((realm, x, z)) in ev_load.iter() {
        println!("Loaded ({:?}, {}, {})", realm, x, z);
        let ent = commands.spawn_bundle(SpriteBundle {
            texture: images.add(render(*realm, *x, *z, &world, &soil_color)),
            transform: Transform::from_translation(
                Vec3::new(*x as f32, *z as f32, 0.) * CHUNK_S1 as f32,
            ),
            ..default()
        });
        col_ents.insert((*realm, *x, *z), ent.id());
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut col_ents: ResMut<HashMap<(Realm, i32, i32), Entity>>,
) {
    for col_ev in ev_unload.iter() {
        if let Some(ent) = col_ents.remove(&col_ev.0) {
            commands.entity(ent).despawn();
        }
    }
}

pub struct SoilColor(HashMap<Bloc, Rgb>);

impl SoilColor {
    pub fn from_csv(path: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut data = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let color = Rgb::from_hex_str(&record[1])?;
            data.insert(Bloc::from_str(&record[0]).unwrap(), color);
        }
        Ok(SoilColor(data))
    }
}
pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoilColor::from_csv("assets/data/soils_color.csv").unwrap())
            .insert_resource(HashMap::<(Realm, i32, i32), Entity>::new())
            .add_startup_system(setup)
            .add_system(update_cam)
            .add_system(on_col_load)
            .add_system(on_col_unload);
    }
}
