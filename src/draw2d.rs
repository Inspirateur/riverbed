use crate::bloc::Bloc;
use crate::chunk::{Chunk, CHUNK_S1, CHUNK_S2};
use crate::load_cols::{ColLoadEvent, ColUnloadEvent};
use crate::player::Dir;
use crate::pos::Pos;
use crate::realm::Realm;
use crate::world_data::{WorldData, WATER_H};
use anyhow::Result;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::texture::BevyDefault;
use colorsys::{Rgb, ColorTransform};
use itertools::iproduct;
use leafwing_input_manager::prelude::ActionState;
use std::cmp::Ordering;
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
    player_query: Query<&Pos, (With<ActionState<Dir>>, Changed<Pos>)>,
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_pos) = player_query.get_single() {
            cam_pos.translation.x = player_pos.coord.x;
            cam_pos.translation.y = player_pos.coord.z;
        }
    }
}

fn top_bloc(col: &[Option<Chunk>], dx: usize, dz: usize) -> (Bloc, i32) {
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

fn bloc_y_cmp(world: &WorldData, realm: Realm, x: i32, y: i32, z: i32, dir: Dir) -> Ordering {
    let dir = Vec2::from(dir);
    let (ox, oz) = (x+dir.x as i32, z+dir.y as i32);
    if world.get(realm, ox, y+1, oz) != Bloc::Air {
        Ordering::Less
    } else if world.get(realm, ox, y, oz) != Bloc::Air {
        Ordering::Equal
    } else {
        Ordering::Greater
    }
}

fn bloc_shade(world: &WorldData, realm: Realm, x: i32, y: i32, z: i32) -> f64 {
    let up_cmp = bloc_y_cmp(world, realm, x, y, z, Dir::Up);
    if up_cmp == Ordering::Greater {
        10.
    } else if up_cmp == Ordering::Less {
        -10.
    } else {
        0.
    }
}

pub fn render(realm: Realm, cx: i32, cz: i32, world: &WorldData, soil_color: &SoilColor) -> Image {
    let mut data = vec![255; CHUNK_S2*4];
    let col = world.chunks.col(realm, cx, cz);
    let def_color = Rgb::default();
    for i in (0..CHUNK_S2 * 4).step_by(4) {
        let (dx, dz) = ((i/4) % CHUNK_S1, CHUNK_S1-1-(i/4) / CHUNK_S1);
        let (bloc, y) = top_bloc(col, dx, dz);
        let color = if y > WATER_H {
            let mut color = soil_color.0.get(&bloc).unwrap_or(&def_color).clone();
            let (x, z) = (cx*CHUNK_S1 as i32+dx as i32, cz*CHUNK_S1 as i32+dz as i32);
            color.lighten(bloc_shade(world, realm, x, y, z));
            color
        } else {
            Rgb::new(10., 180., 250., None)
        };
        data[i] = color.blue() as u8;
        data[i + 1] = color.green() as u8;
        data[i + 2] = color.red() as u8;
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
