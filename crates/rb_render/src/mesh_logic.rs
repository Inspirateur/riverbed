use bevy::{
    asset::RenderAssetUsages,
    log::info_span,
    mesh::{Indices, MeshVertexAttribute},
    prelude::Mesh,
    render::render_resource::{PrimitiveTopology, VertexFormat},
};
use binary_greedy_meshing as bgm;

use super::texture_array::TextureMapTrait;
use rb_block::{Block, Face};
use rb_world::CHUNK_S1;
use rb_world::{Chunk, ChunkPos, WATER_H, linearize};

const MASK_XYZ: u64 = 0b111111_111111_111111;
/// ## Compressed voxel vertex data
/// first u32 (vertex dependant):
///     - in-chunk position: 3x6 bits (18)
///     - texture coords: 2x6 bits (12)
///     - ambiant occlusion?: 2 bits
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
use super::mesh_utils::FaceVertices;

fn color(r: f32, g: f32, b: f32) -> u32 {
    ((r * 63.) as u32) << 11 | ((g * 63.) as u32) << 5 | (b * 31.) as u32
}

pub trait ChunkMeshing {
    fn create_face_meshes(
        &self,
        texture_map: impl TextureMapTrait,
        mesher: &mut bgm::Mesher<CHUNK_S1>,
        voxel_buffer: &mut [u16],
        lod: usize,
        chunk_pos: ChunkPos,
    ) -> [Option<Mesh>; 6];
}

impl ChunkMeshing for Chunk {
    fn create_face_meshes(
        &self,
        texture_map: impl TextureMapTrait,
        mesher: &mut bgm::Mesher<CHUNK_S1>,
        voxel_buffer: &mut [u16],
        lod: usize,
        chunk_pos: ChunkPos,
    ) -> [Option<Mesh>; 6] {
        // palette always has Air at index 0,
        // if the chunk is Air, we can skip it
        if self.palette.len() == 1 {
            return [const { None }; 6];
        }
        let cy = chunk_pos.y as usize * CHUNK_S1 as usize;
        // Gathering binary greedy meshing input data
        let mesh_data_span = info_span!("mesh_logic", name = "mesh voxel data").entered();
        self.data.unpack_u16_into(voxel_buffer);
        mesh_data_span.exit();
        let bgm_mesh_span =
            info_span!("mesh_logic", name = format!("bgm::mesh_lod({lod})")).entered();
        mesher.clear();
        mesher.mesh_lod(voxel_buffer, &self.transparents, lod);
        bgm_mesh_span.exit();
        let mesh_build_span = info_span!("mesh_logic", name = "mesh build").entered();
        let mut meshes = core::array::from_fn(|_| None);
        for (face_n, quads) in mesher.quads.iter().enumerate() {
            let mut voxel_data: Vec<[u32; 2]> = Vec::with_capacity(quads.len() * 4);
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
                let neighbor_block = self.palette[voxel_buffer[linearize(
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
                    _ => (1., 1., 1.),
                };
                if neighbor_block == Block::SeaBlock {
                    let dist_to_surface = (WATER_H as usize - cy - y as usize) as f32;
                    r *= (-dist_to_surface * 0.05).exp();
                    g *= (-dist_to_surface * 0.045).exp();
                    b *= (-dist_to_surface * 0.04).exp();
                }
                let vertices = face.vertices_packed(xyz as u32, w as u32, h as u32, 1);
                let quad_info = (color(r, g, b) << 15) | (layer << 3) | face_n as u32;
                voxel_data.extend_from_slice(&[
                    [vertices[0], quad_info],
                    [vertices[1], quad_info],
                    [vertices[2], quad_info],
                    [vertices[3], quad_info],
                ]);
            }
            if kept_quads == 0 {
                continue;
            }
            let indices = bgm::indices(kept_quads);
            meshes[face_n] = Some(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(ATTRIBUTE_VOXEL_DATA, voxel_data)
                .with_inserted_indices(Indices::U32(indices)),
            )
        }
        mesh_build_span.exit();
        meshes
    }
}
