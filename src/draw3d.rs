use crate::draw2d::{new_tex, update_tex};
use crate::terrain::{Heightmap, Zoom};
use bevy::math::f32;
use bevy::{
    prelude::*,
    render::{camera::Camera, mesh::Indices, render_resource::PrimitiveTopology},
};
use itertools::iproduct;
use std::ops::Rem;
const HEIGHTMULT: f32 = 80.;

fn create_mesh(
    heightmap: Res<Heightmap>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut textures: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the mesh
    let size = heightmap.size;
    let v_pos = iproduct!(0..size, 0..size)
        .map(|(x, y)| [x as f32, 0.0, y as f32])
        .collect::<Vec<[f32; 3]>>();
    let len = v_pos.len();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(
        iproduct!(0..size - 1, 0..size - 1)
            .map(|(x, y)| x + y * size)
            .flat_map(|i| {
                IntoIterator::into_iter([i, i + 1, i + size + 1, i, i + size + 1, i + size])
            })
            .collect(),
    )));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; len]);
    mesh.set_attribute(
        Mesh::ATTRIBUTE_UV_0,
        iproduct!(0..size, 0..size)
            .into_iter()
            .map(|(v, u)| {
                [
                    u as f32 / (size as f32 - 1.0),
                    v as f32 / (size as f32 - 1.0),
                ]
            })
            .collect::<Vec<[f32; 2]>>(),
    );
    let mut tex = new_tex(size as usize, size as usize);
    update_tex(&mut tex.data, &heightmap);
    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(textures.add(tex)),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: material_handle,
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        ..Default::default()
    });
}

fn draw3d(
    heightmap: ResMut<Heightmap>,
    zoom: Res<Zoom>,
    query_mesh: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if heightmap.is_changed() {
        if let Ok(mesh_handle) = query_mesh.get_single() {
            let mesh = &mut *meshes.get_mut(mesh_handle.id).unwrap();
            let v_pos = iproduct!(0..heightmap.size, 0..heightmap.size)
                .map(|(x, y)| {
                    [
                        x as f32,
                        heightmap.data[(y + x * heightmap.size) as usize].max(0.)
                            * HEIGHTMULT
                            * zoom.0,
                        y as f32,
                    ]
                })
                .collect::<Vec<[f32; 3]>>();
            mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos.clone());
        }
    }
}

fn rotate_cam(
    mut query: Query<&mut Transform, With<Camera>>,
    heightmap: Res<Heightmap>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let hsize = (heightmap.size / 2) as f32;
        let alpha = (time.seconds_since_startup() as f32 / 10.).rem(2. * std::f32::consts::PI);
        *transform = Transform::from_xyz(
            hsize + alpha.cos() * hsize,
            hsize,
            hsize + alpha.sin() * hsize,
        )
        .looking_at(Vec3::new(hsize, 0., hsize), Vec3::Y);
    }
}
pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_mesh)
            .add_system(draw3d)
            .add_system(rotate_cam);
    }
}
