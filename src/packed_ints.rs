use intbits::Bits;
// Inspiration from:
// https://github.com/adrianwong/packed-integers/blob/master/src/lib.rs

fn div_ceil(a: u32, b: u32) -> u32 {
    (a as f32 / b as f32).ceil() as u32
}

pub struct PackedUsizes {
    pub len: usize,
    bitsize: u32,
    data: Vec<usize>,
}

impl PackedUsizes {
    pub fn new(len: usize, bitsize: u32) -> Self {
        PackedUsizes {
            len,
            bitsize,
            data: vec![0; div_ceil(bitsize * len as u32, usize::BITS) as usize],
        }
    }

    fn reallocate(&mut self, bitsize: u32) {
        let mut new = PackedUsizes::new(self.len, bitsize);
        for i in 0..self.len {
            new.set(i, self.get(i));
        }
        self.bitsize = new.bitsize;
        self.data = new.data;
    }

    pub fn get(&self, i: usize) -> usize {
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = (start_bit / usize::BITS, start_bit % usize::BITS);
        let end_u = start_u + self.bitsize;
        if end_u <= usize::BITS {
            // the value is in 1 cell
            self.data[index_u as usize].bits(start_u..end_u)
        } else {
            // the value is split in 2 cells
            let end_u = end_u - usize::BITS;
            let mut res = 0;
            res.set_bits(
                ..(self.bitsize - end_u),
                self.data[index_u as usize].bits(start_u..usize::BITS),
            );
            res.set_bits(
                (self.bitsize - end_u)..,
                self.data[1 + index_u as usize].bits(..end_u),
            );
            res
        }
    }

    pub fn set(&mut self, i: usize, value: usize) {
        if value >= 2_usize.pow(self.bitsize) {
            self.reallocate(self.bitsize * 2);
        }
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = (start_bit / usize::BITS, start_bit % usize::BITS);
        let end_u = start_u + self.bitsize;
        if end_u <= usize::BITS {
            // the value is in 1 cell
            self.data[index_u as usize].set_bits(start_u..end_u, value);
        } else {
            // the value is split in 2 cells
            let end_u = end_u - usize::BITS;
            self.data[index_u as usize]
                .set_bits(start_u..usize::BITS, value.bits(..(self.bitsize - end_u)));
            self.data[1 + index_u as usize].set_bits(..end_u, value.bits((self.bitsize - end_u)..));
        }
    }
}

mod tests {
    use super::PackedUsizes;
    fn roundtrip(usizes: &mut PackedUsizes, values: &[usize]) {
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
    pub fn test_no_split() {
        // EASY: All values are within one cell
        // holds 8 integers of 5 bits (40 bits total, max = 2^5-1 = 31)
        let mut usizes = PackedUsizes::new(8, 5);
        let values = [2, 31, 18, 0, 21, 11, 7, 14];
        roundtrip(&mut usizes, &values);
    }

    #[test]
    pub fn test_split() {
        // MEDIUM: Some values are split between 2 cells
        // holds 8 integers of 10 bits (80 bits total, max = 2^10-1 = 1023)
        let mut usizes = PackedUsizes::new(8, 10);
        let values = [541, 26, 999, 4, 0, 263, 1022, 477];
        roundtrip(&mut usizes, &values);
    }

    #[test]
    pub fn test_reallocation() {
        // HARD: some values exceed the capacity of 2^bitsize-1, need to reallocate
        // holds 8 integers of 10 bits (80 bits total, max = 2^10-1 = 1023)
        let mut usizes = PackedUsizes::new(8, 10);
        let values = [541, 26, 999, 4, 0, 263, 1024, 477];
        roundtrip(&mut usizes, &values);
    }
}
