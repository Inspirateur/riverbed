use block_mesh::ndshape::Shape;

use super::CHUNK_S1;

pub struct YFirstShape {
    pub size1: usize,
    pub size2: usize,
    pub size3: usize,
    pub lod: usize
}

impl YFirstShape {
    pub fn new() -> Self {
        let size1: usize = CHUNK_S1;
        Self {
            size1,
            size2: size1*size1,
            size3: size1*size1*size1,
            lod: 1
        }
    }

    pub fn new_padded(lod: usize) -> Self {
        let size1: usize = CHUNK_S1/lod+2;
        Self {
            size1,
            size2: size1*size1,
            size3: size1*size1*size1,
            lod
        }
    }

    #[inline]
    pub fn linearize(&self, x: usize, y: usize, z: usize) -> usize {
        y + z * self.size1 + x * self.size2
    }
}

impl Shape<3> for YFirstShape {
    type Coord = u32;

    fn size(&self) -> Self::Coord {
        self.size3 as u32
    }

    fn usize(&self) -> usize {
        self.size3
    }

    fn as_array(&self) -> [Self::Coord; 3] {
        [self.size1 as u32; 3]
    }

    fn linearize(&self, p: [Self::Coord; 3]) -> Self::Coord {
        // we don't do Y, Z, X here because it doesn't work with the meshing, we invert the axis of the mesh positions instead
        p[0] + p[1] * self.size1 as u32 + p[2] * self.size2 as u32
    }

    fn delinearize(&self, mut _i: Self::Coord) -> [Self::Coord; 3] {
        unimplemented!()
    }
}