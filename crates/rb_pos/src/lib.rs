pub mod pos2d;
pub mod pos3d;
mod realm;
use bevy::math::{I64Vec3, Vec3};
pub use pos2d::*;
pub use pos3d::*;
pub use realm::*;

pub const CHUNK_S1: usize = 62;
pub const REGION_S1: usize = CHUNK_S1 * 16;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNKP_S1: usize = CHUNK_S1 + 2;
pub const CHUNKP_S2: usize = CHUNKP_S1.pow(2);
pub const CHUNKP_S3: usize = CHUNKP_S1.pow(3);
pub const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub const MAX_HEIGHT: usize = 496;
pub const MAX_GEN_HEIGHT: usize = 400;
pub const WATER_H: i32 = 61;
pub const Y_CHUNKS: usize = MAX_HEIGHT / CHUNK_S1;

const CHUNK_S1F: f32 = CHUNK_S1I as f32;

pub fn chunked<const C: usize, const U: usize>(x: i32) -> (i32, usize) {
    let f = (C / U) as i32;
    let r = x.rem_euclid(f);
    ((x - r) / f, r as usize)
}

pub fn unchunked<const C: usize, const U: usize>(cx: i32, dx: usize) -> i32 {
    cx * (C / U) as i32 + dx as i32
}

pub fn chunk_pos(pos: Vec3) -> I64Vec3 {
    I64Vec3::new(
        (pos.x / CHUNK_S1F).floor() as i64,
        (pos.y / CHUNK_S1F).floor() as i64,
        (pos.z / CHUNK_S1F).floor() as i64,
    )
}
