use crate::utils::get_set::GetSet;
use intbits::Bits;
use std::iter;
// Inspiration from:
// https://github.com/adrianwong/packed-integers/blob/master/src/lib.rs

fn div_ceil(a: u32, b: u32) -> u32 {
    (a as f32 / b as f32).ceil() as u32
}

pub fn find_bitsize(value: usize) -> u32 {
    (value.ilog2() + 1).max(4)
}

#[derive(Debug)]
pub struct PackedUsizes {
    pub len: usize,
    bitsize: u32,
    max: usize,
    data: Vec<usize>,
}

fn set_bit(data: &mut Vec<usize>, i: usize, start: u32, end: u32, value: usize) {
    // the value is in 1 cell
    data[i].set_bits(start..end, value);
}

fn get_bit(data: &Vec<usize>, i: usize, start: u32, end: u32) -> usize {
    // the value is in 1 cell
    data[i].bits(start..end)
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
            set_bit(&mut new_data, index_u, start_u, start_u + bitsize, value);
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

impl GetSet<usize> for PackedUsizes {
    fn get(&self, i: usize) -> usize {
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = ((start_bit / usize::BITS) as usize, start_bit % usize::BITS);
        get_bit(&self.data, index_u, start_u, start_u + self.bitsize)
    }

    fn set(&mut self, i: usize, value: usize) {
        if value >= self.max {
            // adding 2 to bitsize multiplies max value by 4
            self.reallocate(find_bitsize(value));
        }
        let start_bit = self.bitsize * i as u32;
        let (index_u, start_u) = ((start_bit / usize::BITS) as usize, start_bit % usize::BITS);
        set_bit(
            &mut self.data,
            index_u,
            start_u,
            start_u + self.bitsize,
            value,
        );
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
            let value = get_bit(
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

#[cfg(test)]
mod tests {
    use crate::utils::get_set::GetSet;

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
    pub fn test_fill() {
        // EASY: Every bit up to the last is used
        // holds 8 integers of 4 bits (32 bits total, max = 2^4-1 = 15)
        let mut usizes = PackedUsizes::new(8, 4);
        roundtrip(&mut usizes, &[2, 15, 6, 0, 4, 5, 10, 11]);
    }

    #[test]
    pub fn test_reallocation() {
        // HARD: some values exceed the capacity of 2^bitsize-1, need to reallocate
        // holds 8 integers of 8 bits (64 bits total, max = 2^8-1 = 255)
        let mut usizes = PackedUsizes::new(8, 8);
        roundtrip(&mut usizes, &[2, 26, 255, 4, 0, 0, 127, 128]);
    }
}
