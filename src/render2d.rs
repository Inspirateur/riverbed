use std::cmp::Ordering;
use bevy::{prelude::Image, render::{render_resource::Extent3d, texture::BevyDefault}};
use colorsys::{Rgb, ColorTransform};
use itertools::iproduct;
use ourcraft::{BlocPos, BlocPos2D, ChunkPos2D, Blocs, Bloc, CHUNK_S2, CHUNK_S1, ChunkPos, ChunkedPos};
use crate::{player::Dir, draw2d::SoilColor, earth_gen::WATER_H};

fn image_to_2d(i: usize) -> (usize, usize) {
    (CHUNK_S1 - 1 - (i / 4) / CHUNK_S1, CHUNK_S1 - 1 - (i / 4) % CHUNK_S1)
}

pub trait ImageUtils {
    fn set_pixel(&mut self, x: i32, z: i32, color: Rgb);
}

impl ImageUtils for Image {
    fn set_pixel(&mut self, x: i32, z: i32, color: Rgb) {
        let i = 4*((CHUNK_S1 - 1 - x as usize)*CHUNK_S1 + CHUNK_S1 - 1 - z as usize);
        self.data[i] = color.red() as u8;
        self.data[i + 1] = color.green() as u8;
        self.data[i + 2] = color.blue() as u8;
    }
}

pub trait Render2D {
    fn bloc_y_cmp(&self, pos: BlocPos, dir: Dir) -> Ordering;
    fn bloc_shade(&self, pos: BlocPos) -> f64;
    fn bloc_color(&self, pos: BlocPos2D, soil_color: &SoilColor) -> Rgb;
    fn update_side(&self, image: &mut Image, col: ChunkPos2D, soil_color: &SoilColor);
    fn render_col(&self, col: ChunkPos2D, soil_color: &SoilColor) -> Image;
    fn process_changes(&mut self, chunk: ChunkPos, changes: Vec<(ChunkedPos, Bloc)>, image: &mut Image, soil_color: &SoilColor);
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
        for (i, (dx, dz)) in iproduct!((0..CHUNK_S1).rev(), (0..CHUNK_S1).rev()).enumerate() {
            let i = i*4;
            let color = self.bloc_color(
                BlocPos2D::from((col, (dx, dz))),
                soil_color,
            );
            data[i] = color.red() as u8;
            data[i + 1] = color.green() as u8;
            data[i + 2] = color.blue() as u8;
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
            let (dx, dz) = image_to_2d(i);
            let color = self.bloc_color(
                BlocPos2D::from((col, (dx, dz))),
                soil_color,
            );
            image.data[i] = color.red() as u8;
            image.data[i + 1] = color.green() as u8;
            image.data[i + 2] = color.blue() as u8;

        }
    }

    fn process_changes(&mut self, chunk: ChunkPos, changes: Vec<(ChunkedPos, Bloc)>, image: &mut Image, soil_color: &SoilColor) {
        for (blocpos, _) in changes {
            let bloc_pos = (chunk.into(), (blocpos.0, blocpos.2)).into();
            let color = self.bloc_color(bloc_pos, &soil_color);
            image.set_pixel(blocpos.0 as i32, blocpos.2 as i32, color);    
        }
    }
}