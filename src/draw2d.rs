use crate::terrain::Earth;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use bevy::render::texture::BevyDefault;

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

pub fn update_tex(tex_data: &mut [u8], earth: &Earth) {
    for (i, v) in earth.elevation.iter().enumerate() {
        tex_data[i * 4 + 3] = 255;
        if *v < -0.4 {
            tex_data[i * 4] = 60;
            tex_data[i * 4 + 1] = 30;
            tex_data[i * 4 + 2] = 10;
        } else if *v < -0.1 {
            tex_data[i * 4] = 120;
            tex_data[i * 4 + 1] = 60;
            tex_data[i * 4 + 2] = 20;
        } else if *v < 0.0 {
            tex_data[i * 4] = 200;
            tex_data[i * 4 + 1] = 100;
            tex_data[i * 4 + 2] = 40;
        } else if *v < 0.02 {
            tex_data[i * 4] = 120;
            tex_data[i * 4 + 1] = 180;
            tex_data[i * 4 + 2] = 200;
        } else if *v < 0.15 {
            let vu = ((*v + 0.1).sqrt() * 255.) as u8;
            tex_data[i * 4] = vu / 2;
            tex_data[i * 4 + 1] = vu;
            tex_data[i * 4 + 2] = vu / 3;
        } else {
            let vu = ((*v).sqrt() * 255.) as u8;
            tex_data[i * 4] = vu;
            tex_data[i * 4 + 1] = vu;
            tex_data[i * 4 + 2] = vu;
        }
    }
}

fn draw2d(query: Query<&Handle<Image>>, earth: Res<Earth>, mut textures: ResMut<Assets<Image>>) {
    if let Ok(im_handle) = query.get_single() {
        let data = &mut *textures.get_mut(im_handle.id).unwrap().data;
        update_tex(data, &earth);
    }
}

pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_tex).add_system(draw2d);
    }
}
