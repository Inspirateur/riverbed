use super::{
    chunked, pos2d::chunks_in_col, BlockPos, BlockPos2d, Chunk, ChunkPos, ChunkedPos, ColPos,
    ColedPos, Realm, CHUNK_S1, MAX_HEIGHT, Y_CHUNKS,
};
use crate::{block::Face, world::CHUNKP_S1, Block};
use bevy::prelude::{Resource, Vec3};
use crossbeam::channel::Sender;
use crossbeam_skiplist::SkipMap;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct BlockRayCastHit {
    pub pos: BlockPos,
    pub normal: Vec3,
}

impl PartialEq for BlockRayCastHit {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

#[derive(Resource, Clone)]
pub struct VoxelWorld {
    pub chunks: Arc<SkipMap<ChunkPos, RwLock<Chunk>>>,
    chunk_changes: Sender<ChunkPos>,
}

impl VoxelWorld {
    pub fn new(chunk_changes: Sender<ChunkPos>) -> Self {
        VoxelWorld {
            chunks: Arc::new(SkipMap::new()),
            chunk_changes,
        }
    }

    pub fn set_block(&self, pos: BlockPos, block: Block) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        self.chunks.get_or_insert_with(chunk_pos, || RwLock::new(Chunk::new()))
            .value()
            .write()
            .set(chunked_pos, block);
        self.mark_change(chunk_pos, chunked_pos, block);
    }

    pub fn set_block_safe(&self, pos: BlockPos, block: Block) -> bool {
        if pos.y < 0 || pos.y >= MAX_HEIGHT as i32 {
            return false;
        }
        self.set_block(pos, block);
        true
    }

    pub fn set_yrange(
        &self,
        col_pos: ColPos,
        (x, z): ColedPos,
        top: i32,
        mut height: usize,
        block: Block,
    ) {
        // USED BY TERRAIN GENERATION - bypasses change detection for efficiency
        let (mut cy, mut dy) = chunked(top);
        while height > 0 && cy >= 0 {
            let chunk_pos = ChunkPos {
                x: col_pos.x,
                y: cy,
                z: col_pos.z,
                realm: col_pos.realm,
            };
            let h = height.min(dy);
            self.chunks.get_or_insert_with(chunk_pos, || RwLock::new(Chunk::new()))
                .value()
                .write()
                .set_yrange((x, dy, z), h, block);
            height -= h;
            cy -= 1;
            dy = CHUNK_S1 - 1;
        }
    }

    pub fn set_if_empty(&self, pos: BlockPos, block: Block) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        if self.chunks.get_or_insert_with(chunk_pos, || RwLock::new(Chunk::new()))
            .value()
            .write()
            .set_if_empty(chunked_pos, block)
        {
            self.mark_change(chunk_pos, chunked_pos, block);
        }
    }

    pub fn get_block(&self, pos: BlockPos) -> Block {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        match self.chunks.get(&chunk_pos) {
            None => Block::Air,
            Some(chunk) => chunk.value().read().get(chunked_pos).clone(),
        }
    }

    pub fn get_block_safe(&self, pos: BlockPos) -> Block {
        if pos.y < 0 || pos.y >= MAX_HEIGHT as i32 {
            Block::Air
        } else {
            self.get_block(pos)
        }
    }

    pub fn top_block(&self, pos: BlockPos2d) -> (Block, i32) {
        let (col_pos, pos2d) = pos.into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk_pos = ChunkPos {
                x: col_pos.x,
                y,
                z: col_pos.z,
                realm: col_pos.realm,
            };
            if let Some(chunk) = self.chunks.get(&chunk_pos) {
                let (&block, block_y) = chunk.value().read().top(pos2d);
                if block != Block::Air {
                    return (block.clone(), y * CHUNK_S1 as i32 + block_y as i32);
                }
            }
        }
        (Block::Air, 0)
    }

    pub fn is_col_loaded(&self, player_pos: Vec3, realm: Realm) -> bool {
        let (chunk_pos, _): (ChunkPos, _) = <BlockPos>::from((player_pos, realm)).into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk = ChunkPos {
                x: chunk_pos.x,
                y,
                z: chunk_pos.z,
                realm: chunk_pos.realm,
            };
            if self.chunks.contains_key(&chunk) {
                return true;
            }
        }
        false
    }

    fn sync_padding_info(&self, chunk_pos: ChunkPos, face: Face) {
        let other_pos = ChunkPos {
            x: chunk_pos.x + face.n()[0],
            y: chunk_pos.y + face.n()[1],
            z: chunk_pos.z + face.n()[2],
            realm: chunk_pos.realm,
        };
        let Some(chunk) = self.chunks.get(&chunk_pos) else {
            return;
        };
        let Some(other) = self.chunks.get(&other_pos) else {
            return;
        };
        chunk.value().write().copy_side_from(&other.value().read(), face);
        other.value().write().copy_side_from(&chunk.value().read(), face.opposite());
        self.chunk_changes.send(other_pos).expect("Failed to send chunk change");
    }

    pub fn mark_change_col(&self, col_pos: ColPos) {
        // USE BY TERRAIN GEN to mass mark change on chunks for efficiency
        for chunk_pos in chunks_in_col(&col_pos) {
            self.sync_padding_info(chunk_pos, Face::Left);
            self.sync_padding_info(chunk_pos, Face::Right);
            self.sync_padding_info(chunk_pos, Face::Front);
            self.sync_padding_info(chunk_pos, Face::Back);
            self.sync_padding_info(chunk_pos, Face::Up);
            // no need to sync down because we're iterating over the column syncing up
            self.chunk_changes.send(chunk_pos).expect("Failed to send chunk change");
        }
    }

    pub fn unload_col(&self, col: ColPos) {
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {
                x: col.x,
                y,
                z: col.z,
                realm: col.realm,
            };
            self.chunks.remove(&chunk_pos);
        }
    }

    fn border_sign(coord: usize) -> i32 {
        if coord == 0 {
            -1
        } else if coord == CHUNK_S1 - 1 {
            1
        } else {
            0
        }
    }

    /// Mark a block change, reflecting in neighboring chunks if needed
    fn mark_change(&self, chunk_pos: ChunkPos, (x, y, z): ChunkedPos, block: Block) {
        self.chunk_changes.send(chunk_pos).expect("Failed to send chunk change");
        // register change for neighboring chunks
        let border_sign_x = VoxelWorld::border_sign(x);
        if border_sign_x != 0 {
            let mut neighbor = chunk_pos;
            neighbor.x += border_sign_x;
            let x = if border_sign_x < 0 { CHUNKP_S1 - 1 } else { 0 };
            if let Some(neighbor_chunk) = self.chunks.get(&neighbor) {
                neighbor_chunk.value().write().set_unpadded((x, y, z), block);
                self.chunk_changes.send(neighbor).expect("Failed to send chunk change");
            }
            // it's possible that other border signs are also != 0 but then we don't care because this means the block is on an edge/corner
            return;
        }
        let border_sign_y = VoxelWorld::border_sign(y);
        if border_sign_y != 0 {
            let mut neighbor = chunk_pos;
            neighbor.y += border_sign_y;
            if neighbor.y >= 0 && neighbor.y < Y_CHUNKS as i32 {
                let y = if border_sign_y < 0 { CHUNKP_S1 - 1 } else { 0 };
                if let Some(neighbor_chunk) = self.chunks.get(&neighbor) {
                    neighbor_chunk.value().write().set_unpadded((x, y, z), block);
                    self.chunk_changes.send(neighbor).expect("Failed to send chunk change");
                }
            }
            return;
        }
        let border_sign_z = VoxelWorld::border_sign(z);
        if border_sign_z != 0 {
            let mut neighbor = chunk_pos;
            neighbor.z += border_sign_z;
            let z = if border_sign_z < 0 { CHUNKP_S1 - 1 } else { 0 };
            if let Some(neighbor_chunk) = self.chunks.get(&neighbor) {
                neighbor_chunk.value().write().set_unpadded((x, y, z), block);
                self.chunk_changes.send(neighbor).expect("Failed to send chunk change");
            }
        }
    }

    pub fn raycast(
        &self,
        realm: Realm,
        start: Vec3,
        dir: Vec3,
        dist: f32,
    ) -> Option<BlockRayCastHit> {
        let mut pos = BlockPos {
            realm,
            x: start.x.floor() as i32,
            y: start.y.floor() as i32,
            z: start.z.floor() as i32,
        };
        let mut last_pos;
        let sx = dir.x.signum() as i32;
        let sy = dir.y.signum() as i32;
        let sz = dir.z.signum() as i32;
        if sx == 0 && sy == 0 && sz == 0 {
            return None;
        }
        let next_x = (pos.x + sx.max(0)) as f32;
        let next_y = (pos.y + sy.max(0)) as f32;
        let next_z = (pos.z + sz.max(0)) as f32;
        let mut t_max_x = (next_x - start.x) / dir.x;
        let mut t_max_y = (next_y - start.y) / dir.y;
        let mut t_max_z = (next_z - start.z) / dir.z;
        let slope_x = 1. / dir.x.abs();
        let slope_y = 1. / dir.y.abs();
        let slope_z = 1. / dir.z.abs();
        loop {
            last_pos = pos;
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x >= dist {
                        return None;
                    };
                    pos.x += sx;
                    t_max_x += slope_x;
                } else {
                    if t_max_z >= dist {
                        return None;
                    };
                    pos.z += sz;
                    t_max_z += slope_z;
                }
            } else if t_max_y < t_max_z {
                if t_max_y >= dist {
                    return None;
                };
                pos.y += sy;
                t_max_y += slope_y;
            } else {
                if t_max_z >= dist {
                    return None;
                };
                pos.z += sz;
                t_max_z += slope_z;
            }
            if self.get_block_safe(pos).is_targetable() {
                return Some(BlockRayCastHit {
                    pos,
                    normal: Vec3 {
                        x: (last_pos.x - pos.x) as f32,
                        y: (last_pos.y - pos.y) as f32,
                        z: (last_pos.z - pos.z) as f32,
                    },
                });
            }
        }
    }
}

