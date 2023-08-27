use std::ops::IndexMut;
use crate::bloc::Bloc;
use crate::MAX_HEIGHT;
use super::{Chunk, CHUNK_S1, pos::{ChunkedPos2D, ColedPos, bloc_pos::chunked}};
use itertools::iproduct;

pub struct Col {
    pub chunks: [Option<Chunk>; MAX_HEIGHT / CHUNK_S1],
}

impl Col {
    pub fn new() -> Self {
        Col {
            chunks: core::array::from_fn(|_| None),
        }
    }

    pub fn top(&self, (x, z): ChunkedPos2D) -> (Bloc, i32) {
        for cy in (0..self.chunks.len()).rev() {
            if let Some(chunk) = &self.chunks[cy] {
                let (bloc, dy) = chunk.top((x, z));
                if *bloc != Bloc::default() {
                    return (*bloc, (cy * CHUNK_S1 + dy) as i32);
                }
            }
        }
        (Bloc::default(), 0)
    }

    pub fn get(&self, (dx, y, dz): ColedPos) -> Bloc {
        let (qy, dy) = chunked(y);
        let qy = qy as usize;
        match &self.chunks[qy] {
            None => Bloc::default(),
            Some(chunk) => chunk.get((dx, dy, dz)).clone()
        }
    }

    pub fn set(&mut self, (dx, y, dz): ColedPos, bloc: Bloc) {
        let (qy, dy) = chunked(y);
        let qy = qy as usize;
        if self.chunks[qy].is_none() {
            self.chunks[qy] = Some(Chunk::new(CHUNK_S1));
        }
        self.chunks[qy].as_mut().unwrap().set((dx, dy, dz), bloc);
    }

    pub fn set_if_empty(&mut self, (dx, y, dz): ColedPos, bloc: Bloc) {
        let (qy, dy) = chunked(y);
        let qy = qy as usize;
        if self.chunks[qy].is_none() {
            self.chunks[qy] = Some(Chunk::new(CHUNK_S1));
        }
        self.chunks[qy].as_mut().unwrap().set_if_empty((dx, dy, dz), bloc);
    }
    
    pub fn fill_up(&mut self, bloc: Bloc) {
        let mut qy = 0;
        // fill the uninitialized chunks
        while self.chunks[qy].is_none() {
            self.chunks[qy] = Some(Chunk::filled(CHUNK_S1, bloc));
            qy += 1;
        }
        // fill the first initialized chunk until the first non-air block (if there's one) 
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let mut dy: usize = 0;
            let chunk = self.chunks.index_mut(qy).as_mut().unwrap();
            while dy < CHUNK_S1 && chunk.set_if_empty((dx, dy, dz), bloc) {
                dy += 1;
            }
        }
    }
}