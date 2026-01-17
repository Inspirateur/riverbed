use std::ops::{Add, BitXor};
use bevy::prelude::Vec3;
use serde::{Deserialize, Serialize};
use crate::world::{Realm, CHUNK_S1};
use super::{chunked, unchunked, ColPos, CHUNK_S1I};

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Default, Debug, Hash, Serialize, Deserialize)]
pub struct Pos3d<const U: usize> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub realm: Realm
}

const K: usize = 0x517cc1b727220a95;

impl<const U: usize> Pos3d<U> {
    pub fn dist(&self, other: Pos3d<U>) -> i32 {
        (self.x - other.x).abs()
            .max((self.y - other.y).abs())
            .max((self.z - other.z).abs())
    }

    fn _prng(&self, seed: usize) -> usize {
        (seed)
            .rotate_left(5).bitxor(self.x as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.y as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.z as usize).wrapping_mul(K)
    }

    pub fn prng(&self, seed: i32) -> usize {
        let n = self._prng(seed as usize);
        self._prng(n)
    }
}

pub type BlockPos = Pos3d<1>;
pub type ChunkPos = Pos3d<CHUNK_S1>;
pub type ChunkedPos = (usize, usize, usize);

impl From<(Vec3, Realm)> for BlockPos {
    fn from((pos, realm): (Vec3, Realm)) -> Self {
        BlockPos {
            x: pos.x.floor() as i32,
            y: pos.y.floor() as i32,
            z: pos.z.floor() as i32,
            realm: realm
        }
    }
}

impl From<BlockPos> for Vec3 {
    fn from(block_pos: BlockPos) -> Self {
        Vec3 {
            x: block_pos.x as f32, 
            y: block_pos.y as f32, 
            z: block_pos.z as f32
        }
    }
}

impl Add<Vec3> for BlockPos {
    type Output = BlockPos;

    fn add(self, rhs: Vec3) -> Self::Output {
        BlockPos {
            x: self.x + rhs.x.floor() as i32,
            y: self.y + rhs.y.floor() as i32,
            z: self.z + rhs.z.floor() as i32,
            realm: self.realm
        }
    }
}

impl Add<(i32, i32, i32)> for BlockPos {
    type Output = BlockPos;

    fn add(self, (dx, dy, dz): (i32, i32, i32)) -> Self::Output {
        BlockPos {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
            realm: self.realm
        }
    }
}

impl From<(ChunkPos, ChunkedPos)> for BlockPos {
    fn from((chunk_pos, (dx, dy, dz)): (ChunkPos, ChunkedPos)) -> Self {
        BlockPos {
            x: unchunked(chunk_pos.x, dx),
            y: unchunked(chunk_pos.y, dy),
            z: unchunked(chunk_pos.z, dz),
            realm: chunk_pos.realm
        }
    }
}

impl From<BlockPos> for (ChunkPos, ChunkedPos) {
    fn from(block_pos: BlockPos) -> Self {
        let (cx, dx) = chunked(block_pos.x);
        let (cy, dy) = chunked(block_pos.y);
        let (cz, dz) = chunked(block_pos.z);
        (ChunkPos {
            x: cx,
            y: cy,
            z: cz,
            realm: block_pos.realm
        }, (dx, dy, dz))
    }
}

impl From<(ColPos, (usize, i32, usize))> for BlockPos {
    fn from((col_pos, (dx, y, dz)): (ColPos, (usize, i32, usize))) -> Self {
        BlockPos {
            x: unchunked(col_pos.x, dx),
            y,
            z: unchunked(col_pos.z, dz),
            realm: col_pos.realm
        }
    }
}

impl From<BlockPos> for (ColPos, (usize, i32, usize)) {
    fn from(block_pos: BlockPos) -> Self {
        let (cx, dx) = chunked(block_pos.x);
        let (cz, dz) = chunked(block_pos.z);
        (ColPos {
            x: cx,
            z: cz,
            realm: block_pos.realm
        }, (dx, block_pos.y, dz))
    }
}

impl From<BlockPos> for ChunkPos {
    fn from(block_pos: BlockPos) -> Self {
        let cx = block_pos.x/CHUNK_S1I;
        let cy = block_pos.y/CHUNK_S1I;
        let cz = block_pos.z/CHUNK_S1I;
        ChunkPos {
            x: cx,
            y: cy,
            z: cz,
            realm: block_pos.realm
        }
    }
}
