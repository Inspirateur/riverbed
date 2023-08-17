use super::pos::{Pos, Pos2D};
const CHUNK_S1I: i32 = 32;

pub type BlocPos = Pos<i32>;
pub type BlocPos2D = Pos2D<i32>;
pub type ChunkPos = Pos<i32>;
pub type ChunkPos2D = Pos2D<i32>;
pub type ChunkedPos = (usize, usize, usize);
pub type ChunkedPos2D = (usize, usize);
pub type ColedPos = (usize, i32, usize);


pub fn chunked(x: i32) -> (i32, usize) {
    let r = x.rem_euclid(CHUNK_S1I);
    ((x - r) / CHUNK_S1I, r as usize)
}

pub fn unchunked(cx: i32, dx: usize) -> i32 {
    cx * CHUNK_S1I + dx as i32
}


impl From<Pos> for ChunkPos2D {
    fn from(pos: Pos) -> Self {
        ChunkPos2D {
            x: chunked(pos.x as i32).0,
            z: chunked(pos.z as i32).0,
        }
    }
}

impl From<BlocPos> for (ChunkPos, ChunkedPos) {
    fn from(pos: BlocPos) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cy, dy) = chunked(pos.y);
        let (cz, dz) = chunked(pos.z);
        (ChunkPos {x: cx, y: cy, z: cz}, (dx, dy, dz))
    }
}

impl From<BlocPos> for (ChunkPos2D, ColedPos) {
    fn from(pos: BlocPos) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cz, dz) = chunked(pos.z);
        (ChunkPos2D {x: cx, z: cz}, (dx, pos.y, dz))
    }
}

impl From<(ChunkPos, ChunkedPos)> for BlocPos {
    fn from((chunk, (dx, dy, dz)): (ChunkPos, ChunkedPos)) -> Self {
        BlocPos {
            x: unchunked(chunk.x, dx),
            y: unchunked(chunk.y, dy),
            z: unchunked(chunk.z, dz),
        }
    }
}

impl From<BlocPos2D> for (ChunkPos2D, ChunkedPos2D) {
    fn from(pos: BlocPos2D) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cz, dz) = chunked(pos.z);
        (ChunkPos2D {x: cx, z: cz}, (dx, dz))
    }
}

impl From<(ChunkPos2D, ChunkedPos2D)> for BlocPos2D {
    fn from((col, (dx, dz)): (ChunkPos2D, ChunkedPos2D)) -> Self {
        BlocPos2D {
            x: unchunked(col.x, dx),
            z: unchunked(col.z, dz),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{BlocPos, ChunkPos, ChunkedPos};
    use super::CHUNK_S1I;

    #[test]
    fn roundtrip() {
        let pos = BlocPos {
            x: -1,
            y: 57,
            z: CHUNK_S1I,
        };
        assert_eq!(pos, BlocPos::from(<(ChunkPos, ChunkedPos)>::from(pos)));
    }
}
