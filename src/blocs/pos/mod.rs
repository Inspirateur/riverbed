pub mod pos3d;
pub mod pos2d;
pub use pos3d::{BlocPos, ChunkPos, ChunkedPos};
pub use pos2d::{BlocPos2d, ColPos, ColedPos};

use super::CHUNK_S1;
const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub fn chunked(x: i32) -> (i32, usize) {
    let r = x.rem_euclid(CHUNK_S1I);
    ((x - r) / CHUNK_S1I, r as usize)
}

pub fn unchunked(cx: i32, dx: usize) -> i32 {
    cx * CHUNK_S1I + dx as i32
}
