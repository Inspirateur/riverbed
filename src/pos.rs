use crate::chunk::CHUNK_S1;
use crate::realm::Realm;
use bevy::{ecs::component::Component, math::Vec3};
use std::ops::{Add, AddAssign};
const CHUNK_S1i: i32 = CHUNK_S1 as i32;
pub trait Fromf32 {
    fn from_f32(v: f32) -> Self;
}

impl Fromf32 for i32 {
    fn from_f32(v: f32) -> Self {
        v as i32
    }
}

impl Fromf32 for f32 {
    fn from_f32(v: f32) -> Self {
        v
    }
}

pub trait Number: Add<Output = Self> + AddAssign + Fromf32 + Sized {}

impl<T> Number for T where T: Add<Output = T> + AddAssign + Fromf32 {}

#[derive(Component, Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct Pos<N: Number = f32, const U: usize = 1> {
    pub realm: Realm,
    pub x: N,
    pub y: N,
    pub z: N,
}

impl<N: Number, V: Into<Vec3>, const U: usize> Add<V> for Pos<N, U> {
    type Output = Pos<N, U>;

    fn add(self, rhs: V) -> Self::Output {
        let rhs = rhs.into();
        Pos {
            realm: self.realm,
            x: self.x + N::from_f32(rhs.x),
            y: self.y + N::from_f32(rhs.y),
            z: self.z + N::from_f32(rhs.z),
        }
    }
}

impl<N: Number, V: Into<Vec3>, const U: usize> AddAssign<V> for Pos<N, U> {
    fn add_assign(&mut self, rhs: V) {
        let rhs = rhs.into();
        self.x += N::from_f32(rhs.x);
        self.y += N::from_f32(rhs.y);
        self.z += N::from_f32(rhs.z);
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Default, Debug)]
pub struct Pos2D<N: Number, const U: usize = 1> {
    pub realm: Realm,
    pub x: N,
    pub z: N,
}

impl<const U: usize> Pos2D<i32, U> {
    pub fn dist(&self, other: Pos2D<i32, U>) -> i32 {
        if self.realm != other.realm {
            i32::MAX
        } else {
            i32::max((self.x-other.x).abs(), (self.z-other.z).abs())
        }
    }
}

impl<N: Number, V: Into<Vec3>, const U: usize> Add<V> for Pos2D<N, U> {
    type Output = Pos2D<N, U>;

    fn add(self, rhs: V) -> Self::Output {
        let rhs = rhs.into();
        Pos2D {
            realm: self.realm,
            x: self.x + N::from_f32(rhs.x),
            z: self.z + N::from_f32(rhs.z),
        }
    }
}

pub type BlocPos = Pos<i32, 1>;
pub type BlocPos2D = Pos2D<i32, 1>;
pub type ChunkPos = Pos<i32, CHUNK_S1>;
pub type ChunkPos2D = Pos2D<i32, CHUNK_S1>;

pub struct BlocPosChunked {
    pub chunk: ChunkPos,
    pub dx: usize,
    pub dy: usize,
    pub dz: usize,
}

impl From<BlocPos> for BlocPosChunked {
    fn from(pos: BlocPos) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cy, dy) = chunked(pos.y);
        let (cz, dz) = chunked(pos.z);
        BlocPosChunked { chunk: ChunkPos {realm: pos.realm, x: cx, y: cy, z: cz}, dx, dy, dz }
    }
}

impl From<BlocPosChunked> for BlocPos {
    fn from(pos: BlocPosChunked) -> Self {
        BlocPos {
            realm: pos.chunk.realm,
            x: pos.chunk.x*CHUNK_S1i+pos.dx as i32,
            y: pos.chunk.y*CHUNK_S1i+pos.dy as i32,
            z: pos.chunk.z*CHUNK_S1i+pos.dz as i32,
        }
    }
}

pub struct BlocPosChunked2D {
    pub col: ChunkPos2D,
    pub dx: usize,
    pub dz: usize,
}

fn chunked(x: i32) -> (i32, usize) {
    let r = x.rem_euclid(CHUNK_S1i);
    ((x-r)/CHUNK_S1i, r as usize)
}

impl From<BlocPos2D> for BlocPosChunked2D {
    fn from(pos: BlocPos2D) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cz, dz) = chunked(pos.z);
        BlocPosChunked2D { col: ChunkPos2D {realm: pos.realm, x: cx, z: cz}, dx, dz }
    }
}

impl From<BlocPosChunked2D> for BlocPos2D {
    fn from(pos: BlocPosChunked2D) -> Self {
        BlocPos2D {
            realm: pos.col.realm,
            x: pos.col.x*CHUNK_S1i+pos.dx as i32,
            z: pos.col.z*CHUNK_S1i+pos.dz as i32,
        }
    }
}

impl From<Pos> for ChunkPos2D {
    fn from(pos: Pos) -> Self {
        ChunkPos2D {
            realm: pos.realm,
            x: chunked(pos.x as i32).0,
            z: chunked(pos.z as i32).0,
        }
    }
}

mod tests {
    use crate::{chunk::CHUNK_S1, realm::Realm, pos::CHUNK_S1i};
    use super::{BlocPos, BlocPosChunked};

    #[test]
    fn roundtrip() {
        let pos = BlocPos {realm: Realm::Earth, x: -1, y: 1, z: CHUNK_S1i};
        assert_eq!(pos, BlocPos::from(BlocPosChunked::from(pos)));
    }
}