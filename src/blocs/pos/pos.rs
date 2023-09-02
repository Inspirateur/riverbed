
use bevy::{ecs::component::Component, prelude::Vec3};
use std::ops::{Add, AddAssign};
use crate::realm::Realm;

pub trait Fromf32 {
    fn from_f32(v: f32) -> Self;
}

impl Fromf32 for i32 {
    fn from_f32(v: f32) -> Self {
        v as i32
    }
}

impl Fromf32 for usize {
    fn from_f32(v: f32) -> Self {
        v as usize
    }
}

impl Fromf32 for f32 {
    fn from_f32(v: f32) -> Self {
        v
    }
}

pub trait Number: Add<Output = Self> + AddAssign + Fromf32 + Sized {}

impl<T> Number for T where T: Add<Output = T> + AddAssign + Fromf32 {}

#[derive(Component)]
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct Pos<N: Number = f32> {
    pub realm: Realm,
    pub x: N,
    pub y: N,
    pub z: N,
}

impl From<Pos<f32>> for Pos<i32> {
    fn from(value: Pos<f32>) -> Self {
        Pos {
            x: value.x.floor() as i32,
            y: value.y.floor() as i32,
            z: value.z.floor() as i32,
            realm: value.realm
        }
    }
}

impl<N: Number> Add<Vec3> for Pos<N> {
    type Output = Pos<N>;

    fn add(self, rhs: Vec3) -> Self::Output {
        Pos {
            realm: self.realm,
            x: self.x + N::from_f32(rhs.x),
            y: self.y + N::from_f32(rhs.y),
            z: self.z + N::from_f32(rhs.z),
        }
    }
}


impl<N: Number, V: Into<Vec3>> AddAssign<V> for Pos<N> {
    fn add_assign(&mut self, rhs: V) {
        let rhs = rhs.into();
        self.x += N::from_f32(rhs.x);
        self.y += N::from_f32(rhs.y);
        self.z += N::from_f32(rhs.z);
    }
}

#[derive(Component)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Default, Debug, PartialOrd, Ord)]
pub struct Pos2D<N: Number> {
    pub realm: Realm,
    pub x: N,
    pub z: N,
}

impl Pos2D<i32> {
    pub fn dist(&self, other: Pos2D<i32>) -> i32 {
        i32::max((self.x - other.x).abs(), (self.z - other.z).abs())
    }
}


impl<N: Number> Add<Vec3> for Pos2D<N> {
    type Output = Pos2D<N>;

    fn add(self, rhs: Vec3) -> Self::Output {
        Pos2D {
            realm: self.realm,
            x: self.x + N::from_f32(rhs.x),
            z: self.z + N::from_f32(rhs.z),
        }
    }
}

impl From<Pos<i32>> for Pos2D<i32> {
    fn from(pos: Pos<i32>) -> Self {
        Pos2D { realm: pos.realm, x: pos.x, z: pos.z }
    }
}