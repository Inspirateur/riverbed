use crate::chunk::CHUNK_S1;
use crate::pos::{Pos, Pos2D};
const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub type BlocPos = Pos<i32>;
pub type BlocPos2D = Pos2D<i32>;
pub type ChunkPos = Pos<i32>;
pub type ChunkPos2D = Pos2D<i32>;

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
        BlocPosChunked {
            chunk: ChunkPos {
                realm: pos.realm,
                x: cx,
                y: cy,
                z: cz,
            },
            dx,
            dy,
            dz,
        }
    }
}

impl From<BlocPosChunked> for BlocPos {
    fn from(pos: BlocPosChunked) -> Self {
        BlocPos {
            realm: pos.chunk.realm,
            x: pos.chunk.x * CHUNK_S1I + pos.dx as i32,
            y: pos.chunk.y * CHUNK_S1I + pos.dy as i32,
            z: pos.chunk.z * CHUNK_S1I + pos.dz as i32,
        }
    }
}

pub struct BlocPosChunked2D {
    pub col: ChunkPos2D,
    pub dx: usize,
    pub dz: usize,
}

fn chunked(x: i32) -> (i32, usize) {
    let r = x.rem_euclid(CHUNK_S1I);
    ((x - r) / CHUNK_S1I, r as usize)
}

impl From<BlocPos2D> for BlocPosChunked2D {
    fn from(pos: BlocPos2D) -> Self {
        let (cx, dx) = chunked(pos.x);
        let (cz, dz) = chunked(pos.z);
        BlocPosChunked2D {
            col: ChunkPos2D {
                realm: pos.realm,
                x: cx,
                z: cz,
            },
            dx,
            dz,
        }
    }
}

impl From<BlocPosChunked2D> for BlocPos2D {
    fn from(pos: BlocPosChunked2D) -> Self {
        BlocPos2D {
            realm: pos.col.realm,
            x: pos.col.x * CHUNK_S1I + pos.dx as i32,
            z: pos.col.z * CHUNK_S1I + pos.dz as i32,
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

#[cfg(test)]
mod tests {
    use super::{BlocPos, BlocPosChunked};
    use crate::{bloc_pos::CHUNK_S1I, realm::Realm};

    #[test]
    fn roundtrip() {
        let pos = BlocPos {
            realm: Realm::Earth,
            x: -1,
            y: 57,
            z: CHUNK_S1I,
        };
        assert_eq!(pos, BlocPos::from(BlocPosChunked::from(pos)));
    }
}
