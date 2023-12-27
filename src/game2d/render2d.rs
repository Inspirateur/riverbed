use std::cmp::Ordering;
use bevy::{prelude::{Image, Vec3}, render::{render_resource::Extent3d, texture::BevyDefault}};
use colorsys::{Rgb, ColorTransform};
use itertools::iproduct;
use crate::blocs::{BlocPos, BlocPos2d, ColPos, Blocs, Bloc, CHUNK_S2, CHUNK_S1};
use crate::agents::Dir;
use super::draw2d::SoilColor;

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
    fn bloc_color(&self, pos: BlocPos2d, soil_color: &SoilColor) -> Rgb;
    fn create_image(&self, col: ColPos, soil_color: &SoilColor) -> Image;
    fn update_image(&self, col: ColPos, image: &mut Image, soil_color: &SoilColor);
    fn update_side(&self, col: ColPos, image: &mut Image, soil_color: &SoilColor);
}

impl Render2D for Blocs {
    fn bloc_y_cmp(&self, pos: BlocPos, dir: Dir) -> Ordering {
        let opos = pos + <Vec3>::from(dir);
        if self.get_block(opos + <Vec3>::from(Dir::Up)) != Bloc::Air {
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

    fn bloc_color(&self, pos: BlocPos2d, soil_color: &SoilColor) -> Rgb {
        let (bloc, y) = self.top_block(pos);
        let mut color = soil_color.0.get(&bloc).unwrap_or(&Rgb::new(1.0, 0.0, 1.0, None)).clone();
        let blocpos = BlocPos {
            realm: pos.realm,
            x: pos.x,
            y,
            z: pos.z,
        };
        color.lighten(self.bloc_shade(blocpos));
        color
    }

    fn create_image(&self, col: ColPos, soil_color: &SoilColor) -> Image {
        let data = vec![255; CHUNK_S2 * 4];
        let mut image = Image::new(
            Extent3d {
                width: CHUNK_S1 as u32,
                height: CHUNK_S1 as u32,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            BevyDefault::bevy_default(),
        );
        self.update_image(col, &mut image, soil_color);
        image
    }

    fn update_image(&self, col: ColPos, image: &mut Image, soil_color: &SoilColor) {
        for (i, (dx, dz)) in iproduct!((0..CHUNK_S1).rev(), (0..CHUNK_S1).rev()).enumerate() {
            let i = i*4;
            let color = self.bloc_color(
                BlocPos2d::from((col, (dx, dz))),
                soil_color,
            );
            image.data[i] = color.red() as u8;
            image.data[i + 1] = color.green() as u8;
            image.data[i + 2] = color.blue() as u8;
        }
    }
    
    fn update_side(&self, col: ColPos, image: &mut Image, soil_color: &SoilColor) {
        for i in (0..CHUNK_S1 * 4).step_by(4) {
            let (dx, dz) = image_to_2d(i);
            let color = self.bloc_color(
                BlocPos2d::from((col, (dx, dz))),
                soil_color,
            );
            image.data[i] = color.red() as u8;
            image.data[i + 1] = color.green() as u8;
            image.data[i + 2] = color.blue() as u8;
        }
    }
}