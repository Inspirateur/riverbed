use std::collections::BTreeSet;
use bevy::log::info_span;
use binary_greedy_meshing::{self as bgm, Face};
use crate::{Block, render::quad_data::QuadData, world::{CHUNKP_S3, Chunk, ChunkPos, WATER_H, linearize, pad_linearize}};
use crate::world::CHUNK_S1;
use super::texture_array::TextureMapTrait;

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
    pub fn create_quads(&self, texture_map: impl TextureMapTrait, lod: usize, chunk_pos: ChunkPos) ->  [Vec<QuadData>; 6] {
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
        let mut meshes = core::array::from_fn(|_| Vec::new());
        for (face_n, quads) in mesher.quads.iter().enumerate() {
            let mut instances: Vec<QuadData> = Vec::with_capacity(quads.len());
            let face: Face = Face::from(face_n as u8);
            for quad in quads {
                let voxel_i = quad.voxel_id() as usize;
                let w = quad.width();
                let h = quad.height();
                let [x, y, z] = quad.xyz();
                let block = self.palette[voxel_i];
                let layer = texture_map.get_texture_index(block, face) as u32;
                let (mut r, mut g, mut b) = match (block, face) {
                    (Block::GrassBlock, Face::Up) => (0.1, 0.9, 0.2),
                    (Block::SeaBlock, _) => (0.1, 0.3, 0.7),
                    (block, _) if block.is_foliage() => (0.1, 0.8, 0.1),
                    _ => (1., 1., 1.)
                };
                instances.push(QuadData::new(
                    x as u32, y as u32, z as u32, 
                    w as u32, h as u32, 
                    layer as u32, 
                    r, g, b
                ));
            }

            meshes[face_n] = instances;
        }
        mesh_build_span.exit();
        meshes
    }
}
