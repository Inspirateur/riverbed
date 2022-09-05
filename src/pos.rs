use crate::realm::Realm;
use crate::chunk::CHUNK_S1;
use bevy::{ecs::component::Component, math::Vec3};
use std::ops::{Add, AddAssign, Mul};

trait Fromf32 {
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

trait Number: Add + Mul + Fromf32 + Sized {}

#[derive(Component, Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct Pos<N: Add + Mul + Fromf32 + Sized=f32, const U: usize=1> {
    pub realm: Realm,
    pub x: N,
    pub y: N,
    pub z: N,
}

impl<N: Add + Mul + Fromf32 + Sized, V: Into<Vec3>, const U: usize> Add<V> for Pos<N, U> {
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
        todo!()
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Default, Debug)]
pub struct Pos2D<N, const U: usize=1> {
    pub realm: Realm,
    pub x: N,
    pub z: N,
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
    fn from(_: BlocPos) -> Self {
        todo!()
    }
}

pub struct BlocPosChunked2D {
    pub col: ChunkPos2D,
    pub dx: usize,
    pub dz: usize,
}

impl From<BlocPos2D> for BlocPosChunked2D {
    fn from(_: BlocPos2D) -> Self {
        todo!()
    }
}

impl From<BlocPosChunked2D> for BlocPos2D {
    fn from(_: BlocPosChunked2D) -> Self {
        todo!()
    }
}