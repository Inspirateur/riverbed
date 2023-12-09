use block_mesh::ndshape::Shape;
use super::{CHUNK_PADDED_S1, CHUNK_PADDED_S2, CHUNK_PADDED_S3, CHUNK_S1, CHUNK_S2, CHUNK_S3};

pub struct YFirstShape<const SIZE1: usize, const SIZE2: usize, const SIZE3: usize> { }

impl<const SIZE1: usize, const SIZE2: usize, const SIZE3: usize> YFirstShape<SIZE1, SIZE2, SIZE3> {
    #[inline]
    pub fn linearize(x: usize, y: usize, z: usize) -> usize {
        y + x * SIZE1 + z * SIZE2
    }
}

impl<const SIZE1: usize, const SIZE2: usize, const SIZE3: usize> Shape<3> for YFirstShape<SIZE1, SIZE2, SIZE3> {
    type Coord = u32;

    fn size(&self) -> Self::Coord {
        SIZE3 as u32
    }

    fn usize(&self) -> usize {
        SIZE3
    }

    fn as_array(&self) -> [Self::Coord; 3] {
        [SIZE3 as u32; 3]
    }

    fn linearize(&self, p: [Self::Coord; 3]) -> Self::Coord {
        YFirstShape::<SIZE1, SIZE2, SIZE3>::linearize(p[0] as usize, p[1] as usize, p[2] as usize) as u32
    }

    fn delinearize(&self, mut i: Self::Coord) -> [Self::Coord; 3] {
        let z = i / SIZE2 as u32;
        i -= z * SIZE2 as u32;
        let x = i / SIZE1 as u32;
        let y = i % SIZE1 as u32;
        [x, y, z]
    }
}

pub type ChunkShape = YFirstShape<CHUNK_S1, CHUNK_S2, CHUNK_S3>;
pub type PaddedChunkShape = YFirstShape<CHUNK_PADDED_S1, CHUNK_PADDED_S2, CHUNK_PADDED_S3>;
