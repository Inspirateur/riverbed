use crate::conditionned_index::ConditionnedIndex;
use crate::draw2d::{new_tex, update_tex};
use crate::terrain::{Earth, Zoom};
use bevy::math::f32;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use itertools::iproduct;
pub const HEIGHTMULT: f32 = 1000.;

fn create_mesh(
    earth: Res<Earth>,
    mut commands: Commands,
    soils: Res<ConditionnedIndex<[u8; 3], 2>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut textures: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hsize = (earth.size / 2) as f32;
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(2.4 * hsize, 1.5 * hsize, hsize)
            .looking_at(Vec3::new(1.3 * hsize, 0., hsize), Vec3::Y),
        ..Default::default()
    });
    // Create the mesh
    let size = earth.size;
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
    update_tex(&mut tex.data, &earth, &soils);
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
}

fn draw3d(
    earth: ResMut<Earth>,
    zoom: Res<Zoom>,
    query_mesh: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if earth.is_changed() {
        if let Ok(mesh_handle) = query_mesh.get_single() {
            let mesh = &mut *meshes.get_mut(mesh_handle.id).unwrap();
            let v_pos = iproduct!(0..earth.size, 0..earth.size)
                .map(|(x, y)| {
                    [
                        x as f32,
                        earth.elevation[(y + x * earth.size) as usize].max(0.)
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

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_mesh).add_system(draw3d);
    }
}
