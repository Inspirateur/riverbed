use crate::CHUNK_S1;
use crate::bloc::Bloc;
use crate::blocs::{BlocPos, ChunkPos2D, BlocPos2D, Blocs, CHUNK_S2, Pos};
use crate::earth_gen::WATER_H;
use crate::load_cols::{ColLoadEvent, ColUnloadEvent};
use crate::player::Dir;
use anyhow::Result;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::texture::BevyDefault;
use colorsys::{ColorTransform, Rgb};
use itertools::iproduct;
use leafwing_input_manager::prelude::ActionState;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::zip;
use std::str::FromStr;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
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
            cam_pos.translation.x = player_pos.x;
            cam_pos.translation.y = player_pos.z;
        }
    }
}

trait Render2D {
    fn bloc_y_cmp(&self, pos: BlocPos, dir: Dir) -> Ordering;
    fn bloc_shade(&self, pos: BlocPos) -> f64;
    fn bloc_color(&self, pos: BlocPos2D, soil_color: &SoilColor) -> Rgb;
    fn update_side(&self, image: &mut Image, col: ChunkPos2D, soil_color: &SoilColor);
    fn render_col(&self, col: ChunkPos2D, soil_color: &SoilColor) -> Image;
}

impl Render2D for Blocs {
    fn bloc_y_cmp(&self, pos: BlocPos, dir: Dir) -> Ordering {
        let opos = pos + dir;
        if self.get_block(opos + Dir::Up) != Bloc::Air {
            Ordering::Less
        } else if self.get_block(opos) != Bloc::Air {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }

    fn bloc_shade(&self, pos: BlocPos) -> f64 {
        let up_cmp = self.bloc_y_cmp(pos, Dir::Front);
        if up_cmp == Ordering::Greater {
            10.
        } else if up_cmp == Ordering::Less {
            -10.
        } else {
            0.
        }
    }

    fn bloc_color(&self, pos: BlocPos2D, soil_color: &SoilColor) -> Rgb {
        let (bloc, y) = self.top_block(pos);
        if y >= WATER_H {
            let mut color = soil_color.0.get(&bloc).unwrap().clone();
            let blocpos = BlocPos {
                realm: pos.realm,
                x: pos.x,
                y,
                z: pos.z,
            };
            color.lighten(self.bloc_shade(blocpos));
            color
        } else if y > WATER_H - 15 {
            Rgb::new(10., 180., 250., None)
        } else {
            Rgb::new(5., 150., 230., None)
        }
    }

    fn render_col(&self, col: ChunkPos2D, soil_color: &SoilColor) -> Image {
        let mut data = vec![255; CHUNK_S2 * 4];
        for (i, (dz, dx)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
            let dz = CHUNK_S1-(dz+1);
            let i = i*4;
            let color = self.bloc_color(
                BlocPos2D::from((col, (dx, dz))),
                soil_color,
            );
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

    fn update_side(&self, image: &mut Image, col: ChunkPos2D, soil_color: &SoilColor) {
        for i in (0..CHUNK_S1 * 4).step_by(4) {
            let (dx, dz) = ((i / 4) % CHUNK_S1, CHUNK_S1 - 1 - (i / 4) / CHUNK_S1);
            let color = self.bloc_color(
                BlocPos2D::from((col, (dx, dz))),
                soil_color,
            );
            image.data[i] = color.blue() as u8;
            image.data[i + 1] = color.green() as u8;
            image.data[i + 2] = color.red() as u8;
        }
    }
}

pub fn on_col_load(
    mut commands: Commands,
    mut ev_load: EventReader<ColLoadEvent>,
    blocs: Res<Blocs>,
    soil_color: Res<SoilColor>,
    imquery: Query<&Handle<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut col_ents: ResMut<ColEntities>,
) {
    let cols: Vec<_> = ev_load.iter().map(|col_ev| col_ev.0).collect();
    let mut ents = Vec::new();
    // Add all the rendered columns before registering them
    for col in cols.iter().filter(|col| blocs.0.contains_key(*col)) {
        println!("Loaded ({:?})", col);
        let ent = commands
            .spawn(SpriteBundle {
                texture: images.add(blocs.render_col(*col, &soil_color)),
                transform: Transform::from_translation(
                    Vec3::new(col.x as f32, col.z as f32, -1.) * CHUNK_S1 as f32,
                ),
                ..default()
            })
            .id();
        ents.push(ent);
        // if there was an already loaded col below
        let col_below = *col + Dir::Back;
        if let Some(ent_below) = col_ents.0.get(&col_below) {
            if let Ok(handle) = imquery.get_component::<Handle<Image>>(*ent_below) {
                if let Some(image) = images.get_mut(&handle) {
                    // update the top side shading with the new information
                    blocs.update_side(image, col_below, &soil_color);
                }
            }
        }
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

#[derive(Resource)]
pub struct SoilColor(HashMap<Bloc, Rgb>);

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
pub struct ColEntities(HashMap::<ChunkPos2D, Entity>);

pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoilColor::from_csv("assets/data/soils_color.csv").unwrap())
            .insert_resource(ColEntities(HashMap::<ChunkPos2D, Entity>::new()))
            .add_systems(Startup, setup)
            .add_systems(Update, update_cam)
            .add_systems(Update, on_col_load)
            .add_systems(Update, on_col_unload);
    }
}
