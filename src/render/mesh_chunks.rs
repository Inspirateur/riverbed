use bevy::{
    log::info_span, prelude::Mesh, 
    render::{mesh::{Indices, MeshVertexAttribute}, 
    render_asset::RenderAssetUsages, 
    render_resource::{PrimitiveTopology, VertexFormat}}
};
use block_mesh::{
    greedy_quads, ndshape::Shape, Axis, AxisPermutation, 
    GreedyQuadsBuffer, MergeVoxel, OrientedBlockFace, QuadCoordinateConfig, Voxel, VoxelVisibility
};
use dashmap::DashMap;
use itertools::iproduct;
use crate::blocks::{Block, Face, FaceSpecifier};
use crate::world::{
    ChunkPos, ChunkedPos, ColedPos, TrackedChunk, YFirstShape, CHUNK_PADDED_S1, CHUNK_S1
};
use super::texture_array::TextureMapTrait;

const Y_FIRST_RIGHT_HANDED_Y_UP_CONFIG: QuadCoordinateConfig = QuadCoordinateConfig {
    faces: [
        OrientedBlockFace::new(-1, AxisPermutation::Zxy),
        OrientedBlockFace::new(-1, AxisPermutation::Xzy),
        OrientedBlockFace::new(-1, AxisPermutation::Yzx),
        OrientedBlockFace::new(1, AxisPermutation::Zxy),
        OrientedBlockFace::new(1, AxisPermutation::Xzy),
        OrientedBlockFace::new(1, AxisPermutation::Yzx),
    ],
    u_flip_face: Axis::X,
};

/// ## Compressed voxel vertex data
/// first u32:
///     - chunk position: 3x6 bits (33 values)
///     - normals: 3 bits (6 values)
///     - ambiant occlusion: 2 bits (4 values)
///     - color: 9 bits (3 r, 3 g, 3 b)
/// `0bxxxxxx_yyyyyy_zzzzzz_nnn_ao_ccccccccc`
///
/// second u32:
///     - texture coords: 2x6 bits (33 values)
///     - light level: 4 bits (16 value)
///     - texture layer: 16 bits
///
/// `0buuuuuu_vvvvvv_llll_iiiiiiiiiiiiiiii`
pub const ATTRIBUTE_VOXEL_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("VoxelData", 48757581, VertexFormat::Uint32x2);


impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Block::Air => VoxelVisibility::Empty,
            block if block.is_transluscent() => VoxelVisibility::Translucent,
            _ => VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for Block {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

pub trait Meshable {
    fn copy_column(&self, buffer: &mut [Block], chunk_pos: ChunkPos, coled_pos: ColedPos, lod: usize);

    fn get_block_chunked(&self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos) -> Block;

    fn fill_padded_block_column(&self, buffer: &mut [Block], chunk: ChunkPos, coled_pos: ColedPos, buffer_shape: &YFirstShape);

    fn fill_padded_chunk(&self, buffer: &mut [Block], chunk: ChunkPos, buffer_shape: &YFirstShape);

    fn create_face_meshes(&self, chunk: ChunkPos, texture_map: &DashMap<(Block, FaceSpecifier), usize>, lod: usize) -> [Option<Mesh>; 6];
}

fn block_at_quad_pos(buffer: &[Block], quad_positions: &[[f32; 3]; 4], quad_normal: &[i32; 3], buffer_shape: &YFirstShape) -> Block {
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
    
    buffer[buffer_shape.linearize(x/buffer_shape.lod+1, y/buffer_shape.lod+1, z/buffer_shape.lod+1)]
}


impl Meshable for DashMap<ChunkPos, TrackedChunk> {
    fn copy_column(&self, buffer: &mut [Block], chunk_pos: ChunkPos, (x, z): ColedPos, lod: usize) {
        let Some(chunk) = self.get(&chunk_pos) else {
            return;
        };
        chunk.copy_column(buffer, (x, z), lod);
    }

    fn get_block_chunked(&self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos) -> Block {
        match self.get(&chunk_pos) {
            None => Block::Air,
            Some(chunk) => chunk.get(chunked_pos).clone()
        }
    }

    fn fill_padded_block_column(&self, buffer: &mut [Block], chunk: ChunkPos, (x, z): ColedPos, buffer_shape: &YFirstShape) {
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

    fn fill_padded_chunk(&self, buffer: &mut [Block], chunk: ChunkPos, buffer_shape: &YFirstShape) {
        for (x, z) in iproduct!((0..CHUNK_S1).step_by(buffer_shape.lod), (0..CHUNK_S1).step_by(buffer_shape.lod)) {
            let i = buffer_shape.linearize(x/buffer_shape.lod+1, 0, z/buffer_shape.lod+1);
            self.fill_padded_block_column(&mut buffer[i..], chunk, (x, z), buffer_shape);
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

    fn create_face_meshes(&self, chunk: ChunkPos, texture_map: &DashMap<(Block, FaceSpecifier), usize>, lod: usize) -> [Option<Mesh>; 6] {
        let lodf32 = lod as f32;
        let padded_chunk_shape = YFirstShape::new_padded(lod);
        let mesh_data_span = info_span!("mesh voxel data", name = "mesh voxel data").entered();
        let mut voxels = vec![Block::Air; padded_chunk_shape.usize()];
        self.fill_padded_chunk(&mut voxels, chunk, &padded_chunk_shape);
        mesh_data_span.exit();
        let mesh_build_span = info_span!("mesh build", name = "mesh build").entered();
        let mut buffer = GreedyQuadsBuffer::new(padded_chunk_shape.usize());
        greedy_quads(
            &voxels,
            &padded_chunk_shape,
            [0; 3],
            [(CHUNK_S1/lod) as u32+1; 3],
            &Y_FIRST_RIGHT_HANDED_Y_UP_CONFIG.faces,
            &mut buffer,
        );
        let num_quads = buffer.quads.num_quads();
        let mut indices = Vec::with_capacity(num_quads * 6);
        let mut voxel_data: Vec<[u32; 2]> = Vec::with_capacity(num_quads * 4);

        let res = [0, 1, 2, 3, 4, 5].map(|i| {
            indices.clear();
            voxel_data.clear();
            let n = i as u32;
            let quad_face = &Y_FIRST_RIGHT_HANDED_Y_UP_CONFIG.faces[i];
            let face: Face = i.into();
            let face_normal = face.n();
            for quad in buffer.quads.groups[i].iter(){
                let mesh_positions = &quad_face.quad_mesh_positions(quad.into(), lodf32)
                    .map(|[y, z, x]| [x-lodf32, y-lodf32, z-lodf32]);

                let mut uvs = quad_face.tex_coords(Y_FIRST_RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &quad);
                // TODO: this hideous code is due to https://github.com/bonsairobo/block-mesh-rs/issues/29
                // fixing this would probably require writing meshing code for Y first chunk shape ...
                if face == Face::Left {
                    uvs[0].swap(0, 1);
                    uvs[1].swap(0, 1);
                    uvs[2].swap(0, 1);
                } else if face == Face::Right {
                    uvs.swap(0, 1);
                    uvs.swap(2, 3);
                    uvs[0].swap(0, 1);
                    uvs[1].swap(0, 1);
                    uvs[2].swap(0, 1);
                }
                
                let block = block_at_quad_pos(
                    &voxels,
                    mesh_positions,
                    &face_normal,
                    &padded_chunk_shape
                );
                let color = match (block, face) {
                    (Block::GrassBlock, Face::Up) => 0b011_111_001,
                    (block, _) if block.is_foliage() => 0b010_101_001,
                    _ => 0b111_111_111
                };
                let layer = texture_map.get_texture_index(block, face).unwrap_or(0) as u32;
                let light = 0u32;
                indices.extend_from_slice(&quad_face.quad_mesh_indices(voxel_data.len() as u32));
                voxel_data.extend(mesh_positions.into_iter().zip(uvs.into_iter()).map(|(pos, uv)| {
                    let [x, y, z] = [pos[0] as u32, pos[1] as u32, pos[2] as u32];
                    let [u, v] = uv;
                    let first = x | (y << 6) | (z << 12) | (n << 18) | (color << 23);
                    let u = u as u32;
                    let v = v as u32;
                    let second = u | (v << 6) | (light << 12) | (layer << 16);
                    [first, second]
                }));
            }

            Some(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(ATTRIBUTE_VOXEL_DATA, voxel_data.clone())
                .with_inserted_indices(Indices::U32(indices.clone())),
            )
        });
        mesh_build_span.exit();
        res
    }
}