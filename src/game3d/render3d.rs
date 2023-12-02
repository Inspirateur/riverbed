use bevy::{prelude::Mesh, render::{render_resource::{PrimitiveTopology, VertexFormat}, mesh::{VertexAttributeValues, Indices, MeshVertexAttribute}}};
use block_mesh::{ndshape::{ConstShape, ConstShape3u32}, UnitQuadBuffer, RIGHT_HANDED_Y_UP_CONFIG, visible_block_faces};
use crate::blocs::{Blocs, CHUNK_S1, Bloc, ChunkPos, BlocPos, ChunkedPos, Face};
use super::texture_array::TextureMap;
const CHUNK_S1I: i32 = CHUNK_S1 as i32;
const CHUNK_PADDED: u32 = CHUNK_S1 as u32 + 2;
type ChunkShape = ConstShape3u32<CHUNK_PADDED, CHUNK_PADDED, CHUNK_PADDED>;

pub const ATTRIBUTE_TEXTURE_LAYER: MeshVertexAttribute = MeshVertexAttribute::new(
    "TextureLayer", 48757581, VertexFormat::Uint32
);

pub trait Meshable {
    fn padded_bloc_data(&self, pos: ChunkPos) -> [Bloc; ChunkShape::SIZE as usize];

    fn create_mesh(&self, chunk: ChunkPos, texture_map: &TextureMap) -> Mesh;

    fn update_mesh(
        &self, chunk: ChunkPos, mesh: &mut Mesh, texture_map: &TextureMap
    );
}

fn chunked_face_pos(quad_positions: &[[f32; 3]; 4], quad_normal: &[i32; 3]) -> (ChunkedPos, Face) {
    let face_delta = [
        quad_normal[0].max(0) as usize, 
        quad_normal[1].max(0) as usize, 
        quad_normal[2].max(0) as usize,
    ];
    let min_face_pos = quad_positions.iter().fold(
        [f32::INFINITY, f32::INFINITY, f32::INFINITY], 
        |mut acc, elem|  {
            acc[0] = acc[0].min(elem[0]);
            acc[1] = acc[1].min(elem[1]);
            acc[2] = acc[2].min(elem[2]);
            acc
        }
    );

    let chunked_pos = (
        min_face_pos[0] as usize - face_delta[0]-1,
        min_face_pos[1] as usize - face_delta[1]-1,
        min_face_pos[2] as usize - face_delta[2]-1,
    );
    let bloc_face = match quad_normal {
        [-1, 0, 0] => Face::Left,
        [1, 0, 0] => Face::Right,
        [0, -1, 0] => Face::Down,
        [0, 1, 0] => Face::Up,
        [0, 0, -1] => Face::Front,
        [0, 0, 1] => Face::Back,
        _ => unreachable!()
    };
    (chunked_pos, bloc_face)
}

impl Meshable for Blocs {
    fn padded_bloc_data(&self, pos: ChunkPos) -> [Bloc; ChunkShape::SIZE as usize] {
        // TODO: make this faster with ndcopy
        let mut voxels = [Bloc::Air; ChunkShape::SIZE as usize];
        for i in 0..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            let y = y as i32 + pos.y*CHUNK_S1I -1;
            if y >= 0 {
                voxels[i as usize] = self.get_block(BlocPos {
                    x: x as i32 + pos.x*CHUNK_S1I -1, 
                    y, 
                    z: z as i32 + pos.z*CHUNK_S1I -1, 
                    realm: pos.realm
                });
            }
        }
        voxels
    }

    fn create_mesh(&self, chunk: ChunkPos, texture_map: &TextureMap) -> Mesh {
        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        self.update_mesh(chunk, &mut render_mesh, texture_map);
        render_mesh
    }

    fn update_mesh(
        &self, chunk: ChunkPos, mesh: &mut Mesh, texture_map: &TextureMap
    ) {
        let voxels = self.padded_bloc_data(chunk);
        let mut buffer = UnitQuadBuffer::new();
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        visible_block_faces(
            &voxels,
            &ChunkShape {},
            [0; 3],
            [CHUNK_S1 as u32+1; 3],
            &faces,
            &mut buffer
        );
        let num_indices = buffer.num_quads() * 6;
        let num_vertices = buffer.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut uvs = Vec::with_capacity(num_vertices);
        let mut layers = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                let mesh_positions = &face.quad_mesh_positions(&quad.into(), 1.0);
                let mesh_normals = &face.quad_mesh_normals();
                positions.extend_from_slice(mesh_positions);
                normals.extend_from_slice(mesh_normals);
                uvs.extend_from_slice(&face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &quad.into()));
                let (chunked_pos, bloc_face) = chunked_face_pos(
                    mesh_positions, 
                    &[mesh_normals[0][0] as i32, mesh_normals[0][1] as i32, mesh_normals[0][2] as i32]
                );
                let bloc = self.get_block_chunked(chunk, chunked_pos);
                let index = texture_map.get(bloc, bloc_face).unwrap_or(0) as u32;
                layers.extend_from_slice(&[index; 4]);
            }
        }
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL, 
            VertexAttributeValues::Float32x3(normals)
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(uvs),
        );
        mesh.insert_attribute(ATTRIBUTE_TEXTURE_LAYER, layers);
        mesh.set_indices(Some(Indices::U32(indices.clone())));
    }
}