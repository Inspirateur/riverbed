use intbits::Bits;
use serde::{Deserialize, Serialize};
use std::iter;
// Inspiration from:
// https://github.com/adrianwong/packed-integers/blob/master/src/lib.rs

fn div_ceil(a: u32, b: u32) -> u32 {
    (a as f32 / b as f32).ceil() as u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackedUsizes {
    pub len: usize,
    bitsize: u32,
    max: usize,
    data: Vec<usize>,
}

fn set_bits(data: &mut Vec<usize>, i: usize, start: u32, end: u32, value: usize) {
    if end <= usize::BITS {
        // the value is in 1 cell
        data[i].set_bits(start..end, value);
    } else {
        // the value is split in 2 cells
        let end = end - usize::BITS;
        data[i].set_bits(start..usize::BITS, value.bits(..(usize::BITS - start)));
        data[1 + i].set_bits(..end, value.bits((usize::BITS - start)..));
    }
}

fn get_bits(data: &Vec<usize>, i: usize, start: u32, end: u32) -> usize {
    if end <= usize::BITS {
        // the value is in 1 cell
        data[i].bits(start..end)
    } else {
        // the value is split in 2 cells
        let end = end - usize::BITS;
        data[i]
            .bits(start..usize::BITS)
            .with_bits((usize::BITS - start).., data[1 + i].bits(..end))
    }
}

impl PackedUsizes {
    fn with_brick(len: usize, bitsize: u32, brick: usize) -> Self {
        debug_assert!(bitsize < usize::BITS);
        PackedUsizes {
            len,
            bitsize,
            max: 2_usize.pow(bitsize),
            data: vec![brick; div_ceil(bitsize * len as u32, usize::BITS) as usize],
        }
    }
    pub fn new(len: usize, bitsize: u32) -> Self {
        PackedUsizes::with_brick(len, bitsize, 0)
    }

    pub fn filled(len: usize, bitsize: u32, value: usize) -> Self {
        if usize::BITS % bitsize == 0 {
            let mut brick = 0;
            for i in (0..usize::BITS).step_by(bitsize as usize) {
                brick.set_bits(i..(i + bitsize), value);
            }
            PackedUsizes::with_brick(len, bitsize, brick)
        } else {
            PackedUsizes::from_iter(iter::repeat(value).take(len), len, bitsize)
        }
    }

    pub fn from_iter<I>(data: I, len: usize, bitsize: u32) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        let mut index_u = 0;
        let mut start_u = 0;
        let mut new_data = vec![0; div_ceil(bitsize * len as u32, usize::BITS) as usize];
        for value in data.into_iter() {
            set_bits(&mut new_data, index_u, start_u, start_u + bitsize, value);
            start_u += bitsize;
            if start_u >= usize::BITS {
                start_u -= usize::BITS;
                index_u += 1;
            }
        }
        PackedUsizes {
            len,
            bitsize,
            max: 2_usize.pow(bitsize),
            data: new_data,
        }
    }

    pub fn from_usizes(data: Vec<usize>, bitsize: u32) -> Self {
        let len = data.len();
        PackedUsizes::from_iter(data, len, bitsize)
    }

    fn reallocate(&mut self, bitsize: u32) {
        let len = self.len;
        *self = PackedUsizes::from_iter(self.into_iter(), len, bitsize);
    }

    pub fn get(&self, i: usize) -> usize {
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = ((start_bit / usize::BITS) as usize, start_bit % usize::BITS);
        get_bits(&self.data, index_u, start_u, start_u + self.bitsize)
    }

    pub fn set(&mut self, i: usize, value: usize) {
        if value >= self.max {
            // adding 2 to bitsize multiplies max value by 4
            self.reallocate(self.bitsize + 2);
        }
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = ((start_bit / usize::BITS) as usize, start_bit % usize::BITS);
        set_bits(
            &mut self.data,
            index_u,
            start_u,
            start_u + self.bitsize,
            value,
        );
    }

    pub fn into_iter(&self) -> PackedUsizesIter {
        PackedUsizesIter {
            len: self.len,
            count: 0,
            index_u: 0,
            start_u: 0,
            bitsize: self.bitsize,
            data: self.data.clone(),
        }
    }
}

pub struct PackedUsizesIter {
    len: usize,
    count: usize,
    index_u: usize,
    start_u: u32,
    bitsize: u32,
    data: Vec<usize>,
}

impl Iterator for PackedUsizesIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.len {
            None
        } else {
            let value = get_bits(
                &mut self.data,
                self.index_u,
                self.start_u,
                self.start_u + self.bitsize,
            );
            self.start_u += self.bitsize;
            if self.start_u >= usize::BITS {
                self.start_u -= usize::BITS;
                self.index_u += 1;
            }
            self.count += 1;
            Some(value)
        }
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
