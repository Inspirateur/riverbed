use bevy::{prelude::Mesh, render::{render_resource::PrimitiveTopology, mesh::{VertexAttributeValues, Indices}}};
use block_mesh::{ndshape::{ConstShape, ConstShape3u32}, UnitQuadBuffer, RIGHT_HANDED_Y_UP_CONFIG, visible_block_faces};
use crate::blocs::{Blocs, CHUNK_S1, Bloc, ChunkPos, Pos, ChunkedPos};
use super::texture_array::TextureMap;
const CHUNK_S1I: i32 = CHUNK_S1 as i32;
const CHUNK_PADDED: u32 = CHUNK_S1 as u32 + 2;
type ChunkShape = ConstShape3u32<CHUNK_PADDED, CHUNK_PADDED, CHUNK_PADDED>;

pub trait Meshable {
    fn fast_mesh(&self, chunk: ChunkPos, texture_map: &TextureMap) -> Mesh;

    fn process_changes(
        &mut self, chunk: ChunkPos, changes: Vec<(ChunkedPos, Bloc)>, mesh: &mut Mesh, texture_map: &TextureMap
    );
}

impl Meshable for Blocs {
    fn fast_mesh(&self, pos: ChunkPos, texture_map: &TextureMap) -> Mesh {
        let mut voxels = [Bloc::Air; ChunkShape::SIZE as usize];
        for i in 0..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            let y = y as i32 + pos.y*CHUNK_S1I -1;
            if y >= 0 {
                voxels[i as usize] = self.get_block(Pos {
                    x: x as i32 + pos.x*CHUNK_S1I -1, 
                    y, 
                    z: z as i32 + pos.z*CHUNK_S1I -1, 
                    realm: pos.realm
                });
            }
        }
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
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }
        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL, 
            VertexAttributeValues::Float32x3(normals)
        );
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        render_mesh.set_indices(Some(Indices::U32(indices.clone())));
        render_mesh
    }

    fn process_changes(
        &mut self, chunk: ChunkPos, changes: Vec<(ChunkedPos, Bloc)>, mesh: &mut Mesh, texture_map: &TextureMap
    ) {
        let mut voxels = [Bloc::Air; ChunkShape::SIZE as usize];
        for i in 0..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            let y = y as i32 + chunk.y*CHUNK_S1I -1;
            if y >= 0 {
                voxels[i as usize] = self.get_block(Pos {
                    x: x as i32 + chunk.x*CHUNK_S1I -1, 
                    y, 
                    z: z as i32 + chunk.z*CHUNK_S1I -1, 
                    realm: chunk.realm
                });
            }
        }
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
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
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
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        mesh.set_indices(Some(Indices::U32(indices.clone())));
    }
}