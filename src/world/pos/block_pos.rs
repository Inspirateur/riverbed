use std::ops::{Add, BitXor};

use bevy::prelude::Vec3;

use super::pos::{Pos, Pos2D};

pub type BlockPos = Pos<i32>;
pub type BlockPos2D = Pos2D<i32>;
pub type ChunkPos = Pos<i32>;
pub type ColPos = Pos2D<i32>;
pub type ChunkedPos = (usize, usize, usize);
pub type ColedPos = (usize, usize);
pub type ColedPos = (usize, i32, usize);


pub fn chunked(x: i32) -> (i32, usize) {
    let r = x.rem_euclid(CHUNK_S1I);
    ((x - r) / CHUNK_S1I, r as usize)
}

pub fn unchunked(cx: i32, dx: usize) -> i32 {
    cx * CHUNK_S1I + dx as i32
}

impl From<Pos<f32>> for ColPos {
    fn from(pos: Pos) -> Self {
        ColPos {
            realm: pos.realm,
            x: chunked(pos.x.floor() as i32).0,
            z: chunked(pos.z.floor() as i32).0,
        }
    }
}

impl From<BlockPos> for Vec3 {
    fn from(value: BlockPos) -> Self {
        Vec3 { x: value.x as f32, y: value.y as f32, z: value.z as f32 }
    }
}

impl From<BlockPos> for (ChunkPos, ChunkedPos) {
    fn from(pos: BlockPos) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cy, dy) = chunked(pos.y);
        let (cz, dz) = chunked(pos.z);
        (ChunkPos {realm: pos.realm, x: cx, y: cy, z: cz}, (dx, dy, dz))
    }
}

impl From<BlockPos> for (ColPos, ColedPos) {
    fn from(pos: BlockPos) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cz, dz) = chunked(pos.z);
        (ColPos { realm: pos.realm, x: cx, z: cz}, (dx, pos.y, dz))
    }
}

impl From<(ChunkPos, ChunkedPos)> for BlockPos {
    fn from((chunk, (dx, dy, dz)): (ChunkPos, ChunkedPos)) -> Self {
        BlockPos {
            realm: chunk.realm,
            x: unchunked(chunk.x, dx),
            y: unchunked(chunk.y, dy),
            z: unchunked(chunk.z, dz),
        }
    }
}

impl From<BlockPos2D> for (ColPos, ColedPos) {
    fn from(pos: BlockPos2D) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cz, dz) = chunked(pos.z);
        (ColPos { realm: pos.realm, x: cx, z: cz}, (dx, dz))
    }
}

impl From<(ColPos, ColedPos)> for BlockPos2D {
    fn from((col, (dx, dz)): (ColPos, ColedPos)) -> Self {
        BlockPos2D {
            realm: col.realm,
            x: unchunked(col.x, dx),
            z: unchunked(col.z, dz),
        }
    }
}

const K: usize = 0x517cc1b727220a95;

impl Pos<i32> {
    pub fn prng(&self, seed: i32) -> usize {
        (seed as usize)
            .rotate_left(5).bitxor(self.x as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.y as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.z as usize).wrapping_mul(K)
    }
}

impl Add<(i32, i32, i32)> for Pos<i32> {
    type Output = Pos<i32>;

    fn add(self, rhs: (i32, i32, i32)) -> Self::Output {
        Pos {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
            z: self.z + rhs.2,
            realm: self.realm
        }
    }
}


impl Pos2D<i32> {
    pub fn prng(&self, seed: i32) -> usize {
        (seed as usize)
            .rotate_left(5).bitxor(self.x as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.z as usize).wrapping_mul(K)
    }
}

impl Add<(i32, i32)> for Pos2D<i32> {
    type Output = Pos2D<i32>;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Pos2D {
            x: self.x + rhs.0,
            z: self.z + rhs.1,
            realm: self.realm
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::blocks::realm::Realm;
    use super::{BlockPos, ChunkPos, ChunkedPos};
    use super::CHUNK_S1I;

    #[test]
    fn roundtrip() {
        let pos = BlockPos {
            realm: Realm::Overworld,
            x: -1,
            y: 57,
            z: CHUNK_S1I,
        };
        assert_eq!(pos, BlockPos::from(<(ChunkPos, ChunkedPos)>::from(pos)));
    }
}
