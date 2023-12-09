use bevy::{prelude::{Mesh, info_span}, render::{render_resource::{PrimitiveTopology, VertexFormat}, mesh::{VertexAttributeValues, Indices, MeshVertexAttribute}}};
use block_mesh::{UnitQuadBuffer, RIGHT_HANDED_Y_UP_CONFIG, visible_block_faces};
use itertools::iproduct;
use crate::blocs::{Blocs, CHUNK_S1, Bloc, ChunkPos, ChunkedPos, Face, ColedPos, PaddedChunkShape, CHUNK_PADDED_S3, CHUNK_PADDED_S1};
use super::texture_array::TextureMap;

pub const ATTRIBUTE_TEXTURE_LAYER: MeshVertexAttribute = MeshVertexAttribute::new(
    "TextureLayer", 48757581, VertexFormat::Uint32
);

pub trait Meshable {
    fn fill_padded_colum(&self, buffer: &mut [Bloc], chunk: ChunkPos, coled_pos: ColedPos);

    fn padded_bloc_data(&self, chunk: ChunkPos) -> [Bloc; CHUNK_PADDED_S3 as usize];

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
    fn fill_padded_colum(&self, buffer: &mut [Bloc], chunk: ChunkPos, (x, z): ColedPos) {
        let chunk_above = ChunkPos {
            x: chunk.x,
            y: chunk.y+1,
            z: chunk.z,
            realm: chunk.realm
        };
        buffer[0] = self.get_block_chunked(chunk_above, (x, 0, z));
        self.copy_column(&mut buffer[1..], chunk, (x, z));
        if chunk.y == 0 { return; }
        let chunk_below = ChunkPos {
            x: chunk.x,
            y: chunk.y-1,
            z: chunk.z,
            realm: chunk.realm
        };
        buffer[CHUNK_S1+1] = self.get_block_chunked(chunk_below, (x, CHUNK_S1-1, z));
    }

    fn padded_bloc_data(&self, chunk: ChunkPos) -> [Bloc; CHUNK_PADDED_S3 as usize] {
        let mut voxels = [Bloc::Air; CHUNK_PADDED_S3 as usize];
        for (x, z) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let i = PaddedChunkShape::linearize(x+1, 0, z+1);
            self.fill_padded_colum(&mut voxels[i..], chunk, (x, z));
        }
        /* TODO: neighbor information seems to mess it up ... figure out why
        let neighbor_front = ChunkPos {
            x: chunk.x,
            y: chunk.y,
            z: chunk.z + 1,
            realm: chunk.realm
        };
        for x in 0..CHUNK_S1 {
            let i = PaddedChunkShape::linearize(x+1, 0, CHUNK_PADDED_S1-1);
            self.fill_padded_colum(&mut voxels[i..], neighbor_front, (x, 0));    
        }
        let neighbor_back = ChunkPos {
            x: chunk.x,
            y: chunk.y,
            z: chunk.z - 1,
            realm: chunk.realm
        };
        for x in 0..CHUNK_S1 {
            let i = PaddedChunkShape::linearize(x+1, 0, 0);
            self.fill_padded_colum(&mut voxels[i..], neighbor_back, (x, CHUNK_S1-1));    
        }
        let neighbor_right = ChunkPos {
            x: chunk.x + 1,
            y: chunk.y,
            z: chunk.z,
            realm: chunk.realm
        };
        for z in 0..CHUNK_S1 {
            let i = PaddedChunkShape::linearize(CHUNK_PADDED_S1-1, 0, z+1);
            self.fill_padded_colum(&mut voxels[i..], neighbor_right, (0, z));    
        }
        let neighbor_left = ChunkPos {
            x: chunk.x - 1,
            y: chunk.y,
            z: chunk.z,
            realm: chunk.realm
        };
        for z in 0..CHUNK_S1 {
            let i = PaddedChunkShape::linearize(0, 0, z+1);
            self.fill_padded_colum(&mut voxels[i..], neighbor_left, (CHUNK_S1-1, z));    
        } */
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
        let mesh_data_span = info_span!("mesh voxel data", name = "mesh voxel data").entered();
        let voxels = self.padded_bloc_data(chunk);
        mesh_data_span.exit();
        let mesh_buil_span = info_span!("mesh build", name = "mesh build").entered();
        let mut buffer = UnitQuadBuffer::new();
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        visible_block_faces(
            &voxels,
            &PaddedChunkShape {},
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
        let mut color = Vec::with_capacity(num_vertices);
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
                color.extend_from_slice(&[match (bloc, bloc_face) {
                    (Bloc::GrassBlock, Face::Up) => [0.2, 0.8, 0.3, 1.0],
                    (bloc, _) if bloc.is_leaves() => [0.1, 0.6, 0.2, 0.5],
                    _ => [1., 1., 1., 1.]
                }; 4]);
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
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(color),
        );
        mesh.insert_attribute(ATTRIBUTE_TEXTURE_LAYER, layers);
        mesh.set_indices(Some(Indices::U32(indices.clone())));
        mesh_buil_span.exit();
    }
}