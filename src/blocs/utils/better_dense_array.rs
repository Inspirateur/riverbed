use crate::blocs::CHUNK_S3;

const U4_IN_USIZE: usize = usize::BITS as usize / 4;
const PARITY_MASK: usize = U4_IN_USIZE - 1;

pub enum BetterPackedData {
    U4([usize; CHUNK_S3 / U4_IN_USIZE]),
    U8([u8; CHUNK_S3]),
    U16([u16; CHUNK_S3]),
    U32([u32; CHUNK_S3]),
}

impl BetterPackedData {
    fn mask(&self) -> usize {
        use BetterPackedData::*;
        match self {
            U4(_) => 15 as usize,
            U8(_) => u8::MAX as usize - 1,
            U16(_) => u16::MAX as usize - 1,
            U32(_) => u32::MAX as usize - 1,
        }
    }

    #[inline(always)]
    fn get(&self, i: usize) -> usize {
        use BetterPackedData::*;
        match self {
            U4(data) => {
                let shift = 4 * (i & PARITY_MASK);
                ((data[i/U4_IN_USIZE] >> shift ) & 0b1111) as usize
            }
            U8(data) => data[i] as usize,
            U16(data) => data[i] as usize,
            U32(data) => data[i] as usize,
        }
    }

    #[inline(always)]
    fn set(&mut self, i: usize, value: usize) {
        use BetterPackedData::*;
        match self {
            U4(data) => {
                let shift: usize = 4 * (i & PARITY_MASK);
                let mask = 0b1111 << shift;
                let i = i / U4_IN_USIZE;
                data[i] &= !mask;
                data[i] |= (value as usize) << shift;
            }
            U8(data) => {
                data[i] = value as u8;
            }
            U16(data) => {
                data[i] = value as u16;
            }
            U32(data) => {
                data[i] = value as u32;
            }
        }
    }
}

pub struct BetterDenseArray {
    pub data: Box<BetterPackedData>,
    pub mask: usize,
}

impl BetterDenseArray {
    pub fn new() -> Self {
        BetterDenseArray {
            data: Box::new(BetterPackedData::U4([0; CHUNK_S3 / U4_IN_USIZE])),
            mask: 0b1111,
        }
    }

    pub fn from(values: &[usize]) -> Self {
        let mut res = BetterDenseArray::new();
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
            use BetterPackedData::*;
            self.data = if bits < 8 {
                Box::<BetterPackedData>::new(U8(core::array::from_fn(|i| self.data.get(i) as u8)))
            } else if bits < 16 {
                Box::<BetterPackedData>::new(U16(core::array::from_fn(|i| self.data.get(i) as u16)))
            } else {
                Box::<BetterPackedData>::new(U32(core::array::from_fn(|i| self.data.get(i) as u32)))
            };
            self.mask = self.data.mask();
        }
        self.data.set(i, value)
    }
}

#[cfg(test)]
mod tests {
    use super::BetterDenseArray;
    use crate::CHUNK_S3;
    use rand::Rng;

    fn roundtrip(usizes: &mut BetterDenseArray, values: &[usize]) {
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
        let mut usizes = BetterDenseArray::new();
        let values: [usize; CHUNK_S3] = [(); CHUNK_S3].map(|_| rng.gen_range(0..16));
        roundtrip(&mut usizes, &values);
    }

    #[test]
    pub fn test_reallocation() {
        // HARD: some values exceed the capacity of 2^bitsize-1, need to reallocate
        let mut rng = rand::thread_rng();
        let mut usizes = BetterDenseArray::new();
        let values: [usize; CHUNK_S3] = [(); CHUNK_S3].map(|_| rng.gen_range(0..u32::MAX) as usize);
        roundtrip(&mut usizes, &values);
    }
}
