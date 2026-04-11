use super::{BlockPos, CHUNK_S1I, ChunkPos, chunked, pos3d::Pos3d, unchunked};
use crate::world::{CHUNK_S1, REGION_S1, Realm, Y_CHUNKS};
use bevy::prelude::Vec3;
use serde::{Deserialize, Serialize};
use std::ops::BitXor;

#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash, Serialize, Deserialize)]
pub struct Pos2d<const U: usize> {
    pub x: i32,
    pub z: i32,
    pub realm: Realm,
}

#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash, Serialize, Deserialize)]
pub struct LocalPos2d<const U: usize> {
    pub x: usize,
    pub z: usize,
}

const K: usize = 0x517cc1b727220a95;

impl<const U: usize> Pos2d<U> {
    pub fn dist(&self, other: Pos2d<U>) -> i32 {
        (self.x - other.x).abs().max((self.z - other.z).abs())
    }

    fn _prng(&self, seed: usize) -> usize {
        (seed)
            .rotate_left(5)
            .bitxor(self.x as usize)
            .wrapping_mul(K)
            .rotate_left(5)
            .bitxor(self.z as usize)
            .wrapping_mul(K)
    }

    pub fn prng(&self, seed: i32) -> usize {
        let n = self._prng(seed as usize);
        self._prng(n)
    }

    pub fn to_real_pos(&self) -> (f32, f32) {
        (
            unchunked::<U, 1>(self.x, 0) as f32,
            unchunked::<U, 1>(self.z, 0) as f32,
        )
    }
}

impl<const U: usize> From<Pos3d<U>> for Pos2d<U> {
    fn from(pos3d: Pos3d<U>) -> Self {
        Pos2d {
            x: pos3d.x,
            z: pos3d.z,
            realm: pos3d.realm,
        }
    }
}

pub type BlockPos2d = Pos2d<1>;
pub type ChunkPos2d = Pos2d<CHUNK_S1>;
pub type ChunkedPos2d = LocalPos2d<CHUNK_S1>;
pub type RegionPos2d = Pos2d<REGION_S1>;
pub type RegionedPos2d = LocalPos2d<REGION_S1>;

impl From<(Vec3, Realm)> for BlockPos2d {
    fn from((pos, realm): (Vec3, Realm)) -> Self {
        BlockPos2d {
            x: pos.x.floor() as i32,
            z: pos.z.floor() as i32,
            realm: realm,
        }
    }
}

impl<const C: usize, const U: usize> From<(Pos2d<C>, LocalPos2d<C>)> for Pos2d<U> {
    fn from((chunk_pos, local_pos): (Pos2d<C>, LocalPos2d<C>)) -> Self {
        Pos2d::<U> {
            x: unchunked::<C, U>(chunk_pos.x, local_pos.x),
            z: unchunked::<C, U>(chunk_pos.z, local_pos.z),
            realm: chunk_pos.realm,
        }
    }
}

impl<const C: usize, const U: usize> From<Pos2d<U>> for (Pos2d<C>, LocalPos2d<C>) {
    fn from(block_pos: Pos2d<U>) -> Self {
        let (cx, dx) = chunked::<C, U>(block_pos.x);
        let (cz, dz) = chunked::<C, U>(block_pos.z);
        (
            Pos2d::<C> {
                x: cx,
                z: cz,
                realm: block_pos.realm,
            },
            LocalPos2d::<C> { x: dx, z: dz },
        )
    }
}

impl From<BlockPos2d> for ChunkPos2d {
    fn from(block_pos2d: BlockPos2d) -> Self {
        let cx = block_pos2d.x.div_euclid(CHUNK_S1I);
        let cz = block_pos2d.z.div_euclid(CHUNK_S1I);
        ChunkPos2d {
            x: cx,
            z: cz,
            realm: block_pos2d.realm,
        }
    }
}

impl From<BlockPos> for ChunkPos2d {
    fn from(block_pos: BlockPos) -> Self {
        let cx = block_pos.x.div_euclid(CHUNK_S1I);
        let cz = block_pos.z.div_euclid(CHUNK_S1I);
        ChunkPos2d {
            x: cx,
            z: cz,
            realm: block_pos.realm,
        }
    }
}

impl From<(Vec3, Realm)> for ChunkPos2d {
    fn from(value: (Vec3, Realm)) -> Self {
        ChunkPos2d::from(BlockPos::from(value))
    }
}

pub fn chunks_in_col(col_pos: &ChunkPos2d) -> [ChunkPos; Y_CHUNKS] {
    std::array::from_fn(|y| ChunkPos {
        x: col_pos.x,
        y: y as i32,
        z: col_pos.z,
        realm: col_pos.realm,
    })
}
