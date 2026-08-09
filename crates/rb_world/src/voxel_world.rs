use crate::{
    BlockPos, CHUNK_S1, CHUNKP_S1, Chunk, ChunkPos, ChunkPos2d, ChunkedPos, Column, MAX_HEIGHT,
    Realm, Y_CHUNKS, chunk, chunks_in_col,
};
use bevy::{
    log::warn,
    prelude::{Resource, Vec3},
};
use crossbeam::channel::Sender;
use crossbeam_skiplist::{SkipMap, SkipSet, map::Entry};
use parking_lot::RwLock;
use rb_block::{Block, Face};
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
    /// Mark columns that eventually should have data
    /// (they may not having it yet because of async loading)
    pub loaded_columns: Arc<SkipSet<ChunkPos2d>>,
    /// Mark columns that eventually shouldn't have data
    /// (they may have it because of structure generation writing to neighboring chunks)
    pub unloaded_columns: Arc<SkipSet<ChunkPos2d>>,
    chunk_changes: Sender<ChunkPos>,
}

impl VoxelWorld {
    pub fn new(chunk_changes: Sender<ChunkPos>) -> Self {
        VoxelWorld {
            chunks: Arc::new(SkipMap::new()),
            loaded_columns: Arc::new(SkipSet::new()),
            unloaded_columns: Arc::new(SkipSet::new()),
            chunk_changes,
        }
    }

    pub fn set_block(&self, pos: BlockPos, block: Block) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        self.chunks
            .get_or_insert_with(chunk_pos, || RwLock::new(Chunk::new()))
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

    pub fn set_if_empty(&self, pos: BlockPos, block: Block) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        if self
            .chunks
            .get_or_insert_with(chunk_pos, || RwLock::new(Chunk::new()))
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

    fn all_neighbor_columns_loaded(&self, col_pos: ChunkPos2d) -> bool {
        [(0, -1), (0, 1), (-1, 0), (1, 0)]
            .iter()
            .map(|(dx, dz)| ChunkPos2d {
                x: col_pos.x + dx,
                z: col_pos.z + dz,
                realm: col_pos.realm,
            })
            .all(|neighbor_col| self.loaded_columns.contains(&neighbor_col))
    }

    fn sync_padding_info(
        &self,
        chunk: &Entry<ChunkPos, RwLock<Chunk>>,
        chunk_pos: ChunkPos,
        face: Face,
    ) {
        let other_pos = ChunkPos {
            x: chunk_pos.x + face.n()[0],
            y: chunk_pos.y + face.n()[1],
            z: chunk_pos.z + face.n()[2],
            realm: chunk_pos.realm,
        };
        let Some(other) = self.chunks.get(&other_pos) else {
            return;
        };
        // Lock both chunks in a fixed order (by position) regardless of which side calls
        // this, otherwise two threads syncing the same pair from opposite ends can each
        // hold one write lock while waiting on the other's read lock (AB-BA deadlock).
        if chunk_pos < other_pos {
            let mut chunk_guard = chunk.value().write();
            let mut other_guard = other.value().write();
            chunk_guard.copy_side_from(&other_guard, face);
            other_guard.copy_side_from(&chunk_guard, face.opposite());
        } else {
            let mut other_guard = other.value().write();
            let mut chunk_guard = chunk.value().write();
            chunk_guard.copy_side_from(&other_guard, face);
            other_guard.copy_side_from(&chunk_guard, face.opposite());
        }
    }

    pub fn add_column(&self, col_pos: ChunkPos2d, column: Column) {
        // USED BY TERRAIN GEN
        let mut cy = -1;
        for chunk in column.0 {
            cy += 1;
            if chunk.is_empty() {
                continue;
            }
            let chunk_pos = ChunkPos {
                x: col_pos.x,
                y: cy,
                z: col_pos.z,
                realm: col_pos.realm,
            };
            let chunk = self.chunks.insert(chunk_pos, RwLock::new(chunk));
            self.sync_padding_info(&chunk, chunk_pos, Face::Left);
            self.sync_padding_info(&chunk, chunk_pos, Face::Right);
            self.sync_padding_info(&chunk, chunk_pos, Face::Front);
            self.sync_padding_info(&chunk, chunk_pos, Face::Back);
            self.sync_padding_info(&chunk, chunk_pos, Face::Up);
            // no need to sync down because we're iterating over the column syncing up
        }
        self.loaded_columns.insert(col_pos);
        // For each synced column (4 neighbors + self), send chunk changes if all neighbors are loaded
        for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0), (0, 0)] {
            let changed_col = ChunkPos2d {
                x: col_pos.x + dx,
                z: col_pos.z + dz,
                realm: col_pos.realm,
            };
            if !self.loaded_columns.contains(&changed_col)
                || !self.all_neighbor_columns_loaded(changed_col)
            {
                continue;
            }
            // send changes for all chunks in the column after every syncing is done
            for chunk_pos in chunks_in_col(&changed_col) {
                self.chunk_changes
                    .send(chunk_pos)
                    .expect("Failed to send chunk change");
            }
        }
    }

    pub fn unload_col(&self, col: ChunkPos2d) {
        self.unloaded_columns.remove(&col);
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
    fn mark_change(&self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos, block: Block) {
        if let Err(_) = self.chunk_changes.send(chunk_pos) {
            warn!("Chunk change channel closed.");
            return;
        }
        // iterates over x, y, z
        for d in 0..3 {
            let border_sign = VoxelWorld::border_sign(chunked_pos[d]);
            if border_sign == 0 {
                continue;
            }
            // the changed block is on the border of a chunk
            let mut neighbor = chunk_pos;
            neighbor[d] += border_sign;
            let c = if border_sign < 0 { CHUNKP_S1 - 1 } else { 0 };
            let mut neighbor_chunked_pos = chunked_pos.clone();
            neighbor_chunked_pos[d] = c;
            let Some(neighbor_chunk) = self.chunks.get(&neighbor) else {
                continue;
            };
            neighbor_chunk
                .value()
                .write()
                .set_unpadded(neighbor_chunked_pos, block);

            if let Err(_) = self.chunk_changes.send(neighbor) {
                warn!("Chunk change channel closed.");
                return;
            }
        }
    }

    pub fn raycast(
        &self,
        realm: Realm,
        start: Vec3,
        dir: Vec3,
        dist: f32,
        grazing: bool,
    ) -> Option<BlockRayCastHit> {
        let mut pos = BlockPos {
            realm,
            x: start.x.floor() as i32,
            y: start.y.floor() as i32,
            z: start.z.floor() as i32,
        };
        let mut last_pos;
        let dir_sign = dir.signum();
        let sx = dir_sign.x as i32;
        let sy = dir_sign.y as i32;
        let sz = dir_sign.z as i32;
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
        let mut grazed_block = None;
        let grazing_dirs = if grazing {
            vec![
                BlockPos {
                    x: -sx,
                    y: 0,
                    z: 0,
                    realm,
                },
                BlockPos {
                    x: 0,
                    y: -sy,
                    z: 0,
                    realm,
                },
                BlockPos {
                    x: 0,
                    y: 0,
                    z: -sz,
                    realm,
                },
            ]
        } else {
            vec![]
        };
        loop {
            last_pos = pos;
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x >= dist {
                        return grazed_block;
                    };
                    pos.x += sx;
                    t_max_x += slope_x;
                } else {
                    if t_max_z >= dist {
                        return grazed_block;
                    };
                    pos.z += sz;
                    t_max_z += slope_z;
                }
            } else if t_max_y < t_max_z {
                if t_max_y >= dist {
                    return grazed_block;
                };
                pos.y += sy;
                t_max_y += slope_y;
            } else {
                if t_max_z >= dist {
                    return grazed_block;
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
            if grazing && grazed_block.is_none() {
                for &grazing_dir in &grazing_dirs {
                    let neighbor = pos + grazing_dir;
                    let neighbor_dir_sign =
                        (<BlockPos as Into<Vec3>>::into(neighbor) - start).signum();
                    // this means that the neighbor face could be pointed at directly
                    // by just changing the direction of the ray (keeping the same start point),
                    // for UX purposes we don't want to consider it as grazing.
                    if neighbor_dir_sign != dir_sign {
                        continue;
                    }
                    if self.get_block_safe(neighbor).is_targetable() {
                        grazed_block = Some(BlockRayCastHit {
                            pos,
                            normal: Vec3::default(),
                        });
                        break;
                    }
                }
            }
        }
    }
}
