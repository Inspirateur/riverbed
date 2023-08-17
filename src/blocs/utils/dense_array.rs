use crate::blocs::CHUNK_S3;
const U4_IN_USIZE: usize = usize::BITS as usize/4;
const PARITY_MASK: usize = U4_IN_USIZE - 1;
pub trait PackedData {
    fn mask(&self) -> usize;

    fn get(&self, i: usize) -> usize;

    fn set(&mut self, i: usize, value: usize);
}

pub struct ArrU4([usize; CHUNK_S3/U4_IN_USIZE]);


impl PackedData for ArrU4 {
    fn mask(&self) -> usize {
        0b1111
    }

    #[inline(always)]
    fn get(&self, i: usize) -> usize {
        let shift = 4 * (i&PARITY_MASK);
        ((self.0[i/U4_IN_USIZE] >> shift ) & 0b1111) as usize
    }

    #[inline(always)]
    fn set(&mut self, i: usize, value: usize) {
        let shift: usize = 4 * (i&PARITY_MASK);
        let mask = 0b1111 << shift;
        let i = i/U4_IN_USIZE;
        self.0[i] &= !mask;
        self.0[i] |= (value as usize) << shift;
    }
}

impl PackedData for [u8; CHUNK_S3] {
    fn mask(&self) -> usize {
        u8::MAX as usize
    }

    fn get(&self, i: usize) -> usize {
        self[i] as usize
    }

    fn set(&mut self, i: usize, value: usize) {
        self[i] = value as u8;
    }
}

impl PackedData for [u16; CHUNK_S3] {
    fn mask(&self) -> usize {
        u16::MAX as usize
    }

    fn get(&self, i: usize) -> usize {
        self[i] as usize
    }

    fn set(&mut self, i: usize, value: usize) {
        self[i] = value as u16;
    }
}

impl PackedData for [u32; CHUNK_S3] {
    fn mask(&self) -> usize {
        u32::MAX as usize
    }

    fn get(&self, i: usize) -> usize {
        self[i] as usize
    }

    fn set(&mut self, i: usize, value: usize) {
        self[i] = value as u32;
    }
}

pub struct DenseArray {
    pub data: Box<dyn PackedData>,
    pub mask: usize
}

impl DenseArray {
    pub fn new() -> Self {
        DenseArray { 
            data: Box::new(ArrU4([0; CHUNK_S3/U4_IN_USIZE])),
            mask: 0b1111
        }
    }

    pub fn from(values: &[usize]) -> Self {
        let mut res = DenseArray::new();
        for (i, value) in values.into_iter().enumerate() {
            res.set(i, *value);
        }
        res
    }

    #[inline]
    pub fn get(&self, i: usize) -> usize {
        self.data.get(i)
    }

    #[inline]
    pub fn set(&mut self, i: usize, value: usize) {
        if value & self.mask != value {
            let bits = value.ilog2();
            self.data = if bits < 8 {
                Box::<[u8; CHUNK_S3]>::new(core::array::from_fn(|i| self.data.get(i) as u8))
            } else if bits < 16 {
                Box::<[u16; CHUNK_S3]>::new(core::array::from_fn(|i| self.data.get(i) as u16))
            } else {
                Box::<[u32; CHUNK_S3]>::new(core::array::from_fn(|i| self.data.get(i) as u32))
            };
        }
        self.mask = self.data.mask();
        self.data.set(i, value)
    }
}


#[cfg(test)]
mod tests {
    use crate::CHUNK_S3;
    use super::DenseArray;
    use rand::Rng;

    fn roundtrip(usizes: &mut DenseArray, values: &[usize]) {
        // store a bunch of values
        for (i, value) in values.iter().enumerate() {
            usizes.set(i, *value);
        }
        // retrieve them and test for equality
        for (i, value) in values.iter().enumerate() {
            assert_eq!(*value, usizes.get(i));
        }
    }

    #[test]
    pub fn test_u4() {
        // EASY: Every values are in range
        // holds integers of 4 bits (max = 2^4-1 = 15)
        let mut rng = rand::thread_rng();
        let mut usizes = DenseArray::new();
        let values: [usize; CHUNK_S3] = [(); CHUNK_S3].map(|_| rng.gen_range(0..16));
        roundtrip(&mut usizes, &values);
    }

    #[test]
    pub fn test_reallocation() {
        // HARD: some values exceed the capacity of 2^bitsize-1, need to reallocate
        let mut rng = rand::thread_rng();
        let mut usizes = DenseArray::new();
        let values: [usize; CHUNK_S3] = [(); CHUNK_S3].map(|_| rng.gen_range(0..u32::MAX) as usize);
        roundtrip(&mut usizes, &values);
    }
}
