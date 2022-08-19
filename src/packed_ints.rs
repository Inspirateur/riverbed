use intbits::Bits;
use serde::{Deserialize, Serialize};
// Inspiration from:
// https://github.com/adrianwong/packed-integers/blob/master/src/lib.rs

fn div_ceil(a: u32, b: u32) -> u32 {
    (a as f32 / b as f32).ceil() as u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackedUsizes {
    pub len: usize,
    bitsize: u32,
    data: Vec<usize>,
}

impl PackedUsizes {
    pub fn new(len: usize, bitsize: u32) -> Self {
        debug_assert!(bitsize < usize::BITS);
        PackedUsizes {
            len,
            bitsize,
            data: vec![0; div_ceil(bitsize * len as u32, usize::BITS) as usize],
        }
    }

    pub fn from_usizes(data: Vec<usize>, bitsize: u32) -> Self {
        let mut index_u = 0;
        let mut start_u = 0;
        let mut packed = PackedUsizes::new(data.len(), bitsize);
        for value in data.into_iter() {
            packed._set(index_u, start_u, value);
            start_u += bitsize;
            if start_u >= usize::BITS {
                start_u -= usize::BITS;
                index_u += 1;
            }
        }
        packed
    }

    fn reallocate(&mut self, bitsize: u32) {
        let mut new = PackedUsizes::new(self.len, bitsize);
        for i in 0..self.len {
            new.set(i, self.get(i));
        }
        self.bitsize = new.bitsize;
        self.data = new.data;
    }

    fn _get(&self, index_u: usize, start_u: u32) -> usize {
        let end_u = start_u + self.bitsize;
        if end_u <= usize::BITS {
            // the value is in 1 cell
            self.data[index_u].bits(start_u..end_u)
        } else {
            // the value is split in 2 cells
            let end_u = end_u - usize::BITS;
            self.data[index_u].bits(start_u..usize::BITS).with_bits(
                (self.bitsize - end_u)..,
                self.data[1 + index_u].bits(..end_u),
            )
        }
    }

    pub fn get(&self, i: usize) -> usize {
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = ((start_bit / usize::BITS) as usize, start_bit % usize::BITS);
        self._get(index_u, start_u)
    }

    fn _set(&mut self, index_u: usize, start_u: u32, value: usize) {
        let end_u = start_u + self.bitsize;
        if end_u <= usize::BITS {
            // the value is in 1 cell
            self.data[index_u].set_bits(start_u..end_u, value);
        } else {
            // the value is split in 2 cells
            let end_u = end_u - usize::BITS;
            self.data[index_u].set_bits(start_u..usize::BITS, value.bits(..(self.bitsize - end_u)));
            self.data[1 + index_u].set_bits(..end_u, value.bits((self.bitsize - end_u)..));
        }
    }

    pub fn set(&mut self, i: usize, value: usize) {
        if value >= 2_usize.pow(self.bitsize) {
            // adding 2 to bitsize multiplies max value by 4
            self.reallocate(self.bitsize + 2);
        }
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = ((start_bit / usize::BITS) as usize, start_bit % usize::BITS);
        self._set(index_u, start_u, value);
    }
}

mod tests {
    use rand::prelude::*;
    use std::time;

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
        roundtrip(&mut usizes, &[2, 31, 18, 0, 21, 11, 7, 14]);
    }

    #[test]
    pub fn test_split() {
        // MEDIUM: Some values are split between 2 cells
        // holds 8 integers of 10 bits (80 bits total, max = 2^10-1 = 1023)
        let mut usizes = PackedUsizes::new(8, 10);
        roundtrip(&mut usizes, &[541, 26, 999, 4, 0, 263, 1022, 477]);
    }

    #[test]
    pub fn test_reallocation() {
        // HARD: some values exceed the capacity of 2^bitsize-1, need to reallocate
        // holds 8 integers of 10 bits (80 bits total, max = 2^10-1 = 1023)
        let mut usizes = PackedUsizes::new(8, 10);
        roundtrip(&mut usizes, &[541, 26, 999, 4, 0, 263, 1024, 477]);
    }

    // actually a benchmark because UGH I don't want to add the whole lib.rs, bench folder with criterion just for this
    // why did they remove the #[bench] macro ?
    #[test]
    pub fn bench_io() {
        let mut rng = rand::thread_rng();
        let n = 1000000;
        let bitsize = 5;
        let mut values = vec![0; n];
        values.fill_with(|| rng.gen_range(0..2_usize.pow(bitsize)));
        // init bench
        let now = time::Instant::now();
        let mut packed = PackedUsizes::from_usizes(values, bitsize);
        let elapsed = now.elapsed();
        println!("Initializing {} values took {} ms", n, elapsed.as_millis());
        let mut indices: Vec<usize> = (0..n).collect();
        indices.shuffle(&mut rng);
        // write bench
        let mut values = vec![0; n];
        values.fill_with(|| rng.gen_range(0..2_usize.pow(bitsize)));
        let now = time::Instant::now();
        for i in indices.iter() {
            packed.set(*i, values[*i]);
        }
        let elapsed = now.elapsed();
        println!("Writing {} values took {} ms", n, elapsed.as_millis());
        // read bench
        let mut values = vec![0; n];
        let now = time::Instant::now();
        for i in indices.into_iter() {
            values[i] = packed.get(i);
        }
        let elapsed = now.elapsed();
        println!("Reading {} values took {} ms", n, elapsed.as_millis());
    }
}
