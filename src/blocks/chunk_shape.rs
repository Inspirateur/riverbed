use block_mesh::ndshape::{RuntimeShape, Shape};
use super::CHUNK_S1;

pub struct YFirstShape {
    shape: RuntimeShape::<u32, 3>,
    pub lod: usize
}

impl YFirstShape {
    pub fn new() -> Self {
        Self {
            shape: RuntimeShape::<u32, 3>::new([CHUNK_S1 as u32; 3]),
            lod: 1
        }
    }

    pub fn new_padded(lod: usize) -> Self {
        let size1: usize = CHUNK_S1/lod+2;
        Self {
            shape: RuntimeShape::<u32, 3>::new([size1 as u32; 3]),
            lod
        }
    }

    pub fn linearize(&self, x: usize, y: usize, z: usize) -> usize {
        self.shape.linearize([y as u32, x as u32, z as u32]) as usize
    }
}

impl Shape<3> for YFirstShape {
    type Coord = u32;

    fn size(&self) -> Self::Coord {
        self.shape.size()
    }

    fn usize(&self) -> usize {
        self.shape.usize()
    }

    fn as_array(&self) -> [Self::Coord; 3] {
        self.shape.as_array()
    }

    fn linearize(&self, [x, y, z]: [Self::Coord; 3]) -> Self::Coord {
        self.shape.linearize([y, x, z])
    }

    fn delinearize(&self, i: Self::Coord) -> [Self::Coord; 3] {
        let [x, y, z] = self.shape.delinearize(i);
        [y, x, z]
    }
}