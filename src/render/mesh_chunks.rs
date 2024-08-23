use std::collections::BTreeSet;

use bevy::{
    log::info_span, prelude::Mesh, 
    render::{mesh::{Indices, MeshVertexAttribute}, 
    render_asset::RenderAssetUsages, 
    render_resource::{PrimitiveTopology, VertexFormat}}
};
use binary_greedy_meshing as bgm;

use dashmap::DashMap;
use crate::{block::{Block, Face, FaceSpecifier}, world::{pad_linearize, Chunk, CHUNKP_S3}};
use crate::world::CHUNK_S1;
use super::texture_array::TextureMapTrait;

const MASK_6: u64 = 0b111111;
const MASK_XYZ: u64 = 0b111111_111111_111111;
/// ## Compressed voxel vertex data
/// first u32 (vertex dependant):
///     - chunk position: 3x6 bits (33 values)
///     - texture coords: 2x6 bits (33 values)
///     - ambiant occlusion: 2 bits (4 values)
/// `0bao_vvvvvv_uuuuuu_zzzzzz_yyyyyy_xxxxxx`
///
/// second u32 (vertex agnostic):
///     - normals: 3 bits (6 values) = face
///     - color: 9 bits (3 r, 3 g, 3 b)
///     - texture layer: 16 bits
///     - light level: 4 bits (16 value)
///
/// `0bllll_iiiiiiiiiiiiiiii_ccccccccc_nnn`
pub const ATTRIBUTE_VOXEL_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelData", 48757581, VertexFormat::Uint32x2);


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
    pub fn create_face_meshes(&self, texture_map: &DashMap<(Block, FaceSpecifier), usize>, lod: usize) ->  [Option<Mesh>; 6] {
        // Gathering binary greedy meshing input data
        let mesh_data_span = info_span!("mesh voxel data", name = "mesh voxel data").entered();
        let voxels = self.voxel_data_lod(lod);
        let mut mesh_data = bgm::MeshData::new();
        mesh_data_span.exit();
        let mesh_build_span = info_span!("mesh build", name = "mesh build").entered();
        let transparents = BTreeSet::from_iter(self.palette.iter().enumerate().filter_map(
            |(i, block)| if i != 0 && !block.is_opaque() {
                Some(i as u16)
            } else {
                None
            }
        ));
        bgm::mesh(&voxels, &mut mesh_data, transparents);
        let mut meshes = core::array::from_fn(|_| None);
        for (face_n, quads) in mesh_data.quads.iter().enumerate() {
            let mut voxel_data: Vec<[u32; 2]> = Vec::with_capacity(quads.len()*4);
            let indices = bgm::indices(quads.len());
            let face: Face = face_n.into();
            for quad in quads {
                let voxel_i = (quad >> 32) as usize;
                let w = MASK_6 & (quad >> 18);
                let h = MASK_6 & (quad >> 24);
                let xyz = MASK_XYZ & quad;
                let block = self.palette[voxel_i];
                let layer = texture_map.get_texture_index(block, face).unwrap_or(0) as u32;
                let color = match (block, face) {
                    (Block::GrassBlock, Face::Up) => 0b011_111_001,
                    (block, _) if block.is_foliage() => 0b010_101_001,
                    _ => 0b111_111_111
                };
                let vertices = face.vertices_packed(xyz as u32, w as u32, h as u32, lod as u32);
                let quad_info = (layer << 12) | (color << 3) | face_n as u32;
                voxel_data.extend_from_slice(&[
                    [vertices[0], quad_info], 
                    [vertices[1], quad_info], 
                    [vertices[2], quad_info], 
                    [vertices[3], quad_info]
                ]);
            }
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
