use bevy::{prelude::{Mesh, info_span}, render::{render_resource::{PrimitiveTopology, VertexFormat}, mesh::{VertexAttributeValues, Indices, MeshVertexAttribute}}};
use block_mesh::{UnitQuadBuffer, RIGHT_HANDED_Y_UP_CONFIG, visible_block_faces};
use dashmap::DashMap;
use itertools::iproduct;
use crate::blocs::{Bloc, ChunkPos, ChunkedPos, ColedPos, Face, TrackedChunk, YFirstShape, CHUNK_PADDED_S1, CHUNK_S1};
use super::texture_array::{FaceSpecifier, TextureMapTrait};

pub const ATTRIBUTE_TEXTURE_LAYER: MeshVertexAttribute = MeshVertexAttribute::new(
    "TextureLayer", 48757581, VertexFormat::Uint32
);

pub trait Meshable {
    fn copy_column(&self, buffer: &mut [Bloc], chunk_pos: ChunkPos, coled_pos: ColedPos, lod: usize);

    fn get_block_chunked(&self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos) -> Bloc;

    fn fill_padded_bloc_column(&self, buffer: &mut [Bloc], chunk: ChunkPos, coled_pos: ColedPos, buffer_shape: &YFirstShape);

    fn fill_padded_chunk(&self, buffer: &mut [Bloc], chunk: ChunkPos, buffer_shape: &YFirstShape);

    fn create_mesh(&self, chunk: ChunkPos, texture_map: &DashMap<(Bloc, FaceSpecifier), usize>, lod: usize) -> Mesh;

    fn update_mesh(
        &self, chunk: ChunkPos, mesh: &mut Mesh, texture_map: &DashMap<(Bloc, FaceSpecifier), usize>, lod: usize
    );
}

fn chunked_face_pos(buffer: &[Bloc], quad_positions: &[[f32; 3]; 4], quad_normal: &[i32; 3], buffer_shape: &YFirstShape) -> (Bloc, Face) {
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

    let (x, y, z) = (
        min_face_pos[0] as usize - face_delta[0],
        min_face_pos[1] as usize - face_delta[1],
        min_face_pos[2] as usize - face_delta[2],
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
    (buffer[buffer_shape.linearize(x+buffer_shape.lod, y+buffer_shape.lod, z+buffer_shape.lod)], bloc_face)
}

impl Meshable for DashMap<ChunkPos, TrackedChunk> {
    fn copy_column(&self, buffer: &mut [Bloc], chunk_pos: ChunkPos, (x, z): ColedPos, lod: usize) {
        let Some(chunk) = self.get(&chunk_pos) else {
            return;
        };
        chunk.copy_column(buffer, (x, z), lod);
    }

    fn get_block_chunked(&self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos) -> Bloc {
        match self.get(&chunk_pos) {
            None => Bloc::default(),
            Some(chunk) => chunk.get(chunked_pos).clone()
        }
    }

    fn fill_padded_bloc_column(&self, buffer: &mut [Bloc], chunk: ChunkPos, (x, z): ColedPos, buffer_shape: &YFirstShape) {
        self.copy_column(&mut buffer[1..], chunk, (x, z), buffer_shape.lod);
        if buffer_shape.lod != 1 { return; }
        let chunk_above = ChunkPos {
            x: chunk.x,
            y: chunk.y+1,
            z: chunk.z,
            realm: chunk.realm
        };
        buffer[CHUNK_S1+1] = self.get_block_chunked(chunk_above, (x, 0, z));
        if chunk.y == 0 { return; }
        let chunk_below = ChunkPos {
            x: chunk.x,
            y: chunk.y-1,
            z: chunk.z,
            realm: chunk.realm
        };
        buffer[0] = self.get_block_chunked(chunk_below, (x, CHUNK_S1-1, z));
    }

    fn fill_padded_chunk(&self, buffer: &mut [Bloc], chunk: ChunkPos, buffer_shape: &YFirstShape) {
        for (x, z) in iproduct!((0..CHUNK_S1).step_by(buffer_shape.lod), (0..CHUNK_S1).step_by(buffer_shape.lod)) {
            let i = buffer_shape.linearize(x/buffer_shape.lod+1, 0, z/buffer_shape.lod+1);
            self.fill_padded_bloc_column(&mut buffer[i..], chunk, (x, z), buffer_shape);
        }
        if buffer_shape.lod != 1 { return; }
        let neighbor_front = ChunkPos {
            x: chunk.x,
            y: chunk.y,
            z: chunk.z + 1,
            realm: chunk.realm
        };
        for x in 0..CHUNK_S1 {
            let i = buffer_shape.linearize(x+1, 1, CHUNK_PADDED_S1-1);
            self.copy_column(&mut buffer[i..], neighbor_front, (x, 0), 1);
        }
        let neighbor_back = ChunkPos {
            x: chunk.x,
            y: chunk.y,
            z: chunk.z - 1,
            realm: chunk.realm
        };
        for x in 0..CHUNK_S1 {
            let i = buffer_shape.linearize(x+1, 1, 0);
            self.copy_column(&mut buffer[i..], neighbor_back, (x, CHUNK_S1-1), 1);    
        }
        let neighbor_right = ChunkPos {
            x: chunk.x + 1,
            y: chunk.y,
            z: chunk.z,
            realm: chunk.realm
        };
        for z in 0..CHUNK_S1 {
            let i = buffer_shape.linearize(CHUNK_PADDED_S1-1, 1, z+1);
            self.copy_column(&mut buffer[i..], neighbor_right, (0, z), 1);    
        }
        let neighbor_left = ChunkPos {
            x: chunk.x - 1,
            y: chunk.y,
            z: chunk.z,
            realm: chunk.realm
        };
        for z in 0..CHUNK_S1 {
            let i = buffer_shape.linearize(0, 1, z+1);
            self.copy_column(&mut buffer[i..], neighbor_left, (CHUNK_S1-1, z), 1);    
        } 
    }

    fn create_mesh(&self, chunk: ChunkPos, texture_map: &DashMap<(Bloc, FaceSpecifier), usize>, lod: usize) -> Mesh {
        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        self.update_mesh(chunk, &mut render_mesh, texture_map, lod);
        render_mesh
    }

    fn update_mesh(
        &self, chunk: ChunkPos, mesh: &mut Mesh, texture_map: &DashMap<(Bloc, FaceSpecifier), usize>, lod: usize
    ) {
        let padded_chunk_shape = YFirstShape::new_padded(lod);
        let mesh_data_span = info_span!("mesh voxel data", name = "mesh voxel data").entered();
        let mut voxels = vec![Bloc::Air; padded_chunk_shape.size3];
        self.fill_padded_chunk(&mut voxels, chunk, &padded_chunk_shape);
        mesh_data_span.exit();
        let mesh_build_span = info_span!("mesh build", name = "mesh build").entered();
        let mut buffer = UnitQuadBuffer::new();
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        visible_block_faces(
            &voxels,
            &padded_chunk_shape,
            [0; 3],
            [(CHUNK_S1/lod) as u32+1; 3],
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
        let lodf32 = lod as f32;
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                let mesh_positions = &face.quad_mesh_positions(&quad.into(), lodf32).map(|[x, y, z]| [x-lodf32, y-lodf32, z-lodf32]);
                let mesh_normals = &face.quad_mesh_normals();
                positions.extend_from_slice(mesh_positions);
                normals.extend_from_slice(mesh_normals);
                uvs.extend_from_slice(&face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &quad.into()));
                let (bloc, bloc_face) = chunked_face_pos(
                    &voxels,
                    mesh_positions, 
                    &[mesh_normals[0][0] as i32, mesh_normals[0][1] as i32, mesh_normals[0][2] as i32],
                    &padded_chunk_shape
                );
                let index = texture_map.get_texture_index(bloc, bloc_face).unwrap_or(0) as u32;
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
        mesh_build_span.exit();
    }
}