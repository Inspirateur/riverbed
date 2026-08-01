use std::ops::{Index, IndexMut};

use rb_world::CHUNK_S1;

#[derive(Default)]
pub struct NoiseSamples {
    slots: Vec<[f32; CHUNK_S1 * CHUNK_S1]>,
    i: usize,
}

impl NoiseSamples {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_slot(&mut self) -> usize {
        if self.i >= self.slots.len() {
            self.slots.push([0.0; CHUNK_S1 * CHUNK_S1]);
        }
        self.i += 1;
        self.i - 1
    }

    pub fn clear(&mut self) {
        self.i = 0;
    }
}

impl Index<usize> for NoiseSamples {
    type Output = [f32; CHUNK_S1 * CHUNK_S1];

    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}

impl IndexMut<usize> for NoiseSamples {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.slots[index]
    }
}
