use crate::realm::Realm;
use bevy::{ecs::component::Component, math::Vec3};
use std::ops::{Deref, DerefMut};

#[derive(Component, PartialEq, Default, Debug)]
pub struct Pos {
    pub coord: Vec3,
    pub realm: Realm,
}

impl Deref for Pos {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.coord
    }
}

impl DerefMut for Pos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.coord
    }
}
