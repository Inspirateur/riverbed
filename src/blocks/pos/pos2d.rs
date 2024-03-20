use std::ops::BitXor;
use bevy::prelude::Vec3;
use crate::blocks::{Realm, CHUNK_S1, Y_CHUNKS};
use super::{chunked, pos3d::Pos3d, unchunked, BlockPos, ChunkPos, CHUNK_S1I};

#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct Pos2d<const U: usize> {
    pub x: i32,
    pub z: i32,
    pub realm: Realm
}

const K: usize = 0x517cc1b727220a95;

impl<const U: usize> Pos2d<U> {
    pub fn dist(&self, other: Pos2d<U>) -> i32 {
        (self.x - other.x).abs()
            .max((self.z - other.z).abs())
    }
    fn _prng(&self, seed: usize) -> usize {
        (seed)
            .rotate_left(5).bitxor(self.x as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.z as usize).wrapping_mul(K)
    }

    pub fn prng(&self, seed: i32) -> usize {
        let n = self._prng(seed as usize);
        self._prng(n)
    }
}

impl<const U: usize> From<Pos3d<U>> for Pos2d<U> {
    fn from(pos3d: Pos3d<U>) -> Self {
        Pos2d { x: pos3d.x, z: pos3d.z, realm: pos3d.realm }
    }
}

pub type BlockPos2d = Pos2d<1>;
pub type ColPos = Pos2d<CHUNK_S1>;
pub type ColedPos = (usize, usize);

impl From<(Vec3, Realm)> for BlockPos2d {
    fn from((pos, realm): (Vec3, Realm)) -> Self {
        BlockPos2d {
            x: pos.x.floor() as i32,
            z: pos.z.floor() as i32,
            realm: realm
        }
    }
}

impl From<(ColPos, ColedPos)> for BlockPos2d {
    fn from((chunk_pos, (dx, dz)): (ColPos, ColedPos)) -> Self {
        BlockPos2d {
            x: unchunked(chunk_pos.x, dx),
            z: unchunked(chunk_pos.z, dz),
            realm: chunk_pos.realm
        }
    }
}

impl From<BlockPos2d> for (ColPos, ColedPos) {
    fn from(block_pos: BlockPos2d) -> Self {
        let (cx, dx) = chunked(block_pos.x);
        let (cz, dz) = chunked(block_pos.z);
        (ColPos {
            x: cx,
            z: cz,
            realm: block_pos.realm
        }, (dx, dz))
    }
}

impl From<BlockPos2d> for ColPos {
    fn from(block_pos2d: BlockPos2d) -> Self {
        let cx = block_pos2d.x/CHUNK_S1I;
        let cz = block_pos2d.z/CHUNK_S1I;
        ColPos {
            x: cx,
            z: cz,
            realm: block_pos2d.realm
        }
    }
}

impl From<BlockPos> for ColPos {
    fn from(block_pos: BlockPos) -> Self {
        let cx = block_pos.x/CHUNK_S1I;
        let cz = block_pos.z/CHUNK_S1I;
        ColPos {
            x: cx,
            z: cz,
            realm: block_pos.realm
        }
    }
}

impl From<(Vec3, Realm)> for ColPos {
    fn from(value: (Vec3, Realm)) -> Self {
        ColPos::from(BlockPos::from(value))
    }
}

pub fn chunks_in_col(col_pos: &ColPos) -> [ChunkPos; Y_CHUNKS] {
    std::array::from_fn(|y| ChunkPos {
        x: col_pos.x,
        y: y as i32,
        z: col_pos.z,
        realm: col_pos.realm
    })
}