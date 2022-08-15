use crate::terrain::Earth;
use crate::weighted_dist::WeightedPoints;
use anyhow::Result;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use bevy::render::texture::BevyDefault;
use colorsys::Rgb;
use itertools::*;
use std::collections::HashMap;

pub fn new_tex(width: usize, height: usize) -> Image {
    Image::new(
        Extent3d {
            width: width as u32,
            height: width as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; (width * height * 4) as usize],
        BevyDefault::bevy_default(),
    )
}

fn create_tex(earth: Res<Earth>, mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let tex = new_tex(earth.size as usize, earth.size as usize);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        texture: textures.add(tex),
        ..Default::default()
    });
}

pub fn update_tex(
    tex_data: &mut [u8],
    earth: &Earth,
    soils: &WeightedPoints<String>,
    colors: &HashMap<String, Rgb>,
) {
    for (i, (y, d, t, h)) in izip!(
        &earth.elevation,
        &earth.slope,
        &earth.temperature,
        &earth.humidity
    )
    .enumerate()
    {
        tex_data[i * 4 + 3] = 255;
        if *y < -0.6 {
            tex_data[i * 4] = 60;
            tex_data[i * 4 + 1] = 30;
            tex_data[i * 4 + 2] = 10;
        } else if *y < -0.2 {
            tex_data[i * 4] = 120;
            tex_data[i * 4 + 1] = 60;
            tex_data[i * 4 + 2] = 20;
        } else if *y < 0.0 {
            tex_data[i * 4] = 200;
            tex_data[i * 4 + 1] = 100;
            tex_data[i * 4 + 2] = 40;
        } else if *y < 0.01 {
            tex_data[i * 4] = 120;
            tex_data[i * 4 + 1] = 200;
            tex_data[i * 4 + 2] = 220;
        } else {
            if (*d).abs() < 0.1 {
                let (soil, _) = soils.closest(&[*t, *h]);
                let color = colors.get(&soil).unwrap();
                tex_data[i * 4] = color.blue() as u8;
                tex_data[i * 4 + 1] = color.green() as u8;
                tex_data[i * 4 + 2] = color.red() as u8;
            } else {
                let vu = ((*y).sqrt() * 255.) as u8;
                tex_data[i * 4] = vu;
                tex_data[i * 4 + 1] = vu;
                tex_data[i * 4 + 2] = vu;
            }
        }
    }
}

fn draw2d(
    query: Query<&Handle<Image>>,
    earth: Res<Earth>,
    soils: Res<WeightedPoints<String>>,
    colors: Res<SoilColor>,
    mut textures: ResMut<Assets<Image>>,
) {
    if earth.is_changed() {
        if let Ok(im_handle) = query.get_single() {
            let data = &mut *textures.get_mut(im_handle.id).unwrap().data;
            update_tex(data, &earth, &soils, &colors.0);
        }
    }
}

pub struct SoilColor(HashMap<String, Rgb>);

impl SoilColor {
    pub fn from_csv(path: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut data = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let color = Rgb::from_hex_str(&record[1])?;
            data.insert(record[0].to_string(), color);
        }
        Ok(SoilColor(data))
    }
}
pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoilColor::from_csv("assets/data/soils_color.csv").unwrap())
            .add_startup_system(create_tex)
            .add_system(draw2d);
    }
}
