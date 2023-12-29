use std::ops::BitXor;
use bevy::prelude::Vec3;
use crate::blocs::{Realm, CHUNK_S1};
use super::{unchunked, chunked, CHUNK_S1I, pos3d::Pos3d, BlocPos};

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

pub type BlocPos2d = Pos2d<1>;
pub type ColPos = Pos2d<CHUNK_S1>;
pub type ColedPos = (usize, usize);

impl From<(Vec3, Realm)> for BlocPos2d {
    fn from((pos, realm): (Vec3, Realm)) -> Self {
        BlocPos2d {
            x: pos.x.floor() as i32,
            z: pos.z.floor() as i32,
            realm: realm
        }
    }
}

impl From<(ColPos, ColedPos)> for BlocPos2d {
    fn from((chunk_pos, (dx, dz)): (ColPos, ColedPos)) -> Self {
        BlocPos2d {
            x: unchunked(chunk_pos.x, dx),
            z: unchunked(chunk_pos.z, dz),
            realm: chunk_pos.realm
        }
    }
}

impl From<BlocPos2d> for (ColPos, ColedPos) {
    fn from(bloc_pos: BlocPos2d) -> Self {
        let (cx, dx) = chunked(bloc_pos.x);
        let (cz, dz) = chunked(bloc_pos.z);
        (ColPos {
            x: cx,
            z: cz,
            realm: bloc_pos.realm
        }, (dx, dz))
    }
}

impl From<BlocPos2d> for ColPos {
    fn from(bloc_pos2d: BlocPos2d) -> Self {
        let cx = bloc_pos2d.x/CHUNK_S1I;
        let cz = bloc_pos2d.z/CHUNK_S1I;
        ColPos {
            x: cx,
            z: cz,
            realm: bloc_pos2d.realm
        }
    }
}

impl From<BlocPos> for ColPos {
    fn from(bloc_pos: BlocPos) -> Self {
        let cx = bloc_pos.x/CHUNK_S1I;
        let cz = bloc_pos.z/CHUNK_S1I;
        ColPos {
            x: cx,
            z: cz,
            realm: bloc_pos.realm
        }
    }
}