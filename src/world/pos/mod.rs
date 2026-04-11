pub mod pos2d;
pub mod pos3d;
use bevy::math::{I64Vec3, Vec3};
pub use pos2d::{BlockPos2d, ChunkPos2d, ChunkedPos2d};
pub use pos3d::{BlockPos, ChunkPos, ChunkedPos};

use super::CHUNK_S1I;
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
