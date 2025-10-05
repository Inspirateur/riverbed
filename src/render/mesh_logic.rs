use std::collections::BTreeSet;

use bevy::{
    asset::RenderAssetUsages, log::info_span, mesh::{Indices, MeshVertexAttribute}, prelude::Mesh, render::render_resource::{PrimitiveTopology, VertexFormat}
};
use binary_greedy_meshing as bgm;

use crate::{block::Face, world::{linearize, pad_linearize, Chunk, ChunkPos, CHUNKP_S3, WATER_H}, Block};
use crate::world::CHUNK_S1;
use super::texture_array::TextureMapTrait;

const MASK_XYZ: u64 = 0b111111_111111_111111;
/// ## Compressed voxel vertex data
/// first u32 (vertex dependant):
///     - chunk position: 3x6 bits (33 values)
///     - texture coords: 2x6 bits (33 values)
///     - ambiant occlusion?: 2 bits (4 values)
/// `0bao_vvvvvv_uuuuuu_zzzzzz_yyyyyy_xxxxxx`
///
/// second u32 (vertex agnostic):
///     - normals: 3 bits (6 values) = face
///     - texture layer: 12 bits
///     - light/color: 17 bits (6 r, 6 g, 5 b)
/// `0bllll_iiiiiiiiiiiiiiii_ccccccccc_nnn`
pub const ATTRIBUTE_VOXEL_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelData", 48757581, VertexFormat::Uint32x2);

/// Map channels between 0.0 and 1.0 to the correct range and pack them
fn color(r: f32, g: f32, b: f32) -> u32 {
    ((r*63.) as u32) << 11 | ((g*63.) as u32) << 5 | (b*31.) as u32
}

impl Chunk {
    pub fn voxel_data_lod(&self, lod: usize) -> Vec<u16> {
        let voxels = self.data.unpack_u16();
        if lod == 1 {
            return voxels;
        }
        let mut res = vec![0; CHUNKP_S3];
        for x in 0..CHUNK_S1 {
            for y in 0..CHUNK_S1 {
                for z in 0..CHUNK_S1 {
                    let lod_i = pad_linearize(x/lod, y/lod, z/lod);        
                    if res[lod_i] == 0 {
                        res[lod_i] = voxels[pad_linearize(x, y, z)];
                    }
                }
            }
        }
        res
    }

    /// Doesn't work with lod > 2, because chunks are of size 62 (to get to 64 with padding) and 62 = 2*31
    /// TODO: make it work with lod > 2 if necessary (by truncating quads)
    pub fn create_face_meshes(&self, texture_map: impl TextureMapTrait, lod: usize, chunk_pos: ChunkPos) ->  [Option<Mesh>; 6] {
        let cy = chunk_pos.y as usize * CHUNK_S1 as usize;
        // Gathering binary greedy meshing input data
        let mesh_data_span = info_span!("mesh voxel data", name = "mesh voxel data").entered();
        let voxels = self.voxel_data_lod(lod);
        let mut mesher: bgm::Mesher<CHUNK_S1> = bgm::Mesher::new();
        mesh_data_span.exit();
        let mesh_build_span = info_span!("mesh build", name = "mesh build").entered();
        let transparents = BTreeSet::from_iter(self.palette.iter().enumerate().filter_map(
            |(i, block)| if i != 0 && !block.is_opaque() {
                Some(i as u16)
            } else {
                None
            }
        ));
        mesher.mesh(&voxels, &transparents);
        let mut meshes = core::array::from_fn(|_| None);
        for (face_n, quads) in mesher.quads.iter().enumerate() {
            let mut voxel_data: Vec<[u32; 2]> = Vec::with_capacity(quads.len()*4);
            let face: Face = face_n.into();
            let offset = face.quad_to_block();
            let mut kept_quads = 0;
            for quad in quads {
                let voxel_i = quad.voxel_id() as usize;
                let w = quad.width();
                let h = quad.height();
                let xyz = MASK_XYZ & quad.0;
                let [x, y, z] = quad.xyz();
                let block = self.palette[voxel_i];
                let neighbor_block = self.palette[voxels[linearize(
                    (offset[0] + x as i32 + 1) as usize,
                    (offset[1] + y as i32 + 1) as usize,
                    (offset[2] + z as i32 + 1) as usize,
                )] as usize];
                kept_quads += 1;
                let layer = texture_map.get_texture_index(block, face) as u32;
                let (mut r, mut g, mut b) = match (block, face) {
                    (Block::GrassBlock, Face::Up) => (0.1, 0.9, 0.2),
                    (Block::SeaBlock, _) => (0.1, 0.3, 0.7),
                    (block, _) if block.is_foliage() => (0.1, 0.8, 0.1),
                    _ => (1., 1., 1.)
                };
                if neighbor_block == Block::SeaBlock {
                    let dist_to_surface = (WATER_H as usize - cy - y as usize) as f32;
                    r *= (-dist_to_surface*0.05).exp();
                    g *= (-dist_to_surface*0.045).exp();
                    b *= (-dist_to_surface*0.04).exp();
                }
                let vertices = face.vertices_packed(xyz as u32, w as u32, h as u32, lod as u32);
                let quad_info = (color(r, g, b) << 15) | (layer << 3) | face_n as u32;
                voxel_data.extend_from_slice(&[
                    [vertices[0], quad_info], 
                    [vertices[1], quad_info], 
                    [vertices[2], quad_info], 
                    [vertices[3], quad_info]
                ]);
            }
            let indices = bgm::indices(kept_quads);
            meshes[face_n] = Some(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(ATTRIBUTE_VOXEL_DATA, voxel_data)
                .with_inserted_indices(Indices::U32(indices)),
            )
        }
        mesh_build_span.exit();
        meshes
    }
}
