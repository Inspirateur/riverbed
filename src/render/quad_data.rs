use std::sync::Arc;
use bevy::{math::Vec3, platform::collections::HashSet, prelude::{Component, Deref}};
use bytemuck::{Pod, Zeroable};
use parking_lot::RwLock;

use crate::world::ChunkPos;

#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct QuadData {
    /// `0btthhhhhhwwwwwwzzzzzzyyyyyyxxxxxx`
    /// - x, y, z origin of quad in chunk: 3x6 bits
    /// - w, h size of quad: 2x6 bits
    /// - 2 bits of texture layer
    pub quad_pos: u32,
    /// `0brrrrrrrggggggggbbbbbbbtttttttttt`
    /// - 10 bits of texture layer
    /// - r, g, b light/color: 24 bits (7, 8, 7)
    pub quad_info: u32,
    // TODO: remove this when a proper draw id is implemented in WGPU
    // https://github.com/gfx-rs/wgpu/issues/6823#issuecomment-3714274449
    pub draw_id: u32,
}

// Maps a float between 0.0 and 1.0 to an N-bit integer
fn float_to_nbit<const N: usize>(value: f32) -> u32 {
    let max_value = (1 << N) - 1;
    (value * (max_value as f32)).round() as u32
}

impl QuadData {
    pub fn new(x: u32, y: u32, z: u32, w: u32, h: u32, t: u32, r: f32, g: f32, b: f32) -> Self {
        let quad_pos = t << 30 | h << 24 | w << 18 | z << 12 | y << 6 | x;
        // we lose a bit of accuracy for red and blue to fit everything in 32 bits
        let r = float_to_nbit::<7>(r);
        let g = float_to_nbit::<8>(g);
        let b = float_to_nbit::<7>(b);
        let quad_info = r << 25 | g << 17 | b << 10 | t >> 2;
        Self {
            quad_pos,
            quad_info,
            // valued only by culling() function in QuadBuffer
            draw_id: 0,
        }
    }
}

#[derive(Default)]
pub struct FaceBuffer {
    pub quads: Vec<QuadData>,
    /// Vec of [[ (chunk_pos, index, count) ]] - the span of quads in the quads buffer for each chunk
    pub chunk_spans: Vec<(ChunkPos, usize, usize)>, 
    /// Vec of [[ (index, count) ]] - the free spans in the quads buffer
    pub free_spans: Vec<(usize, usize)>, 
    pub chunks: HashSet<ChunkPos>,
}

// Chunk buffer associated to faces in the quad buffer
#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkBuffer {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: u32,
}

impl FaceBuffer {
    pub fn add(&mut self, quads: Vec<QuadData>, chunk_pos: ChunkPos) {
        if self.chunks.contains(&chunk_pos) {
            self.remove(chunk_pos);
        }
        if let Some(span_index) = self.free_spans.iter().position(|&(_, count)| count >= quads.len()) {
            let (start, count) = self.free_spans[span_index];
            self.quads[start..start + quads.len()].copy_from_slice(&quads);
            if count == quads.len() {
                self.free_spans.swap_remove(span_index);
            } else {
                self.free_spans[span_index].0 += quads.len();
                self.free_spans[span_index].1 -= quads.len();
            }
            self.chunk_spans.push((chunk_pos, start, quads.len()));
        } else {
            self.chunk_spans.push((chunk_pos, self.quads.len(), quads.len()));
            self.quads.extend_from_slice(&quads);
        }
        self.chunks.insert(chunk_pos);
    }

    pub fn remove(&mut self, chunk_pos: ChunkPos) {
        if !self.chunks.remove(&chunk_pos) {
            return;
        }
        let i = self.chunk_spans.iter().position(|(pos, _, _)| *pos == chunk_pos).unwrap();
        let (_, start, count) = self.chunk_spans.swap_remove(i);
        let end = start + count;
        // Free span merging logic to avoid fragmentation
        let lower_free_span_idx = self.free_spans.iter().position(|&(s, c)| s + c == start);
        let upper_free_span_idx = self.free_spans.iter().position(|&(s, _)| s == end);
        match (lower_free_span_idx, upper_free_span_idx) {
            (Some(lower_idx), Some(upper_idx)) => {
                let (_, upper_count) = self.free_spans[upper_idx];
                self.free_spans[lower_idx].1 += count + upper_count;
                self.free_spans.swap_remove(upper_idx);
            },
            (Some(lower_idx), None) => {
                self.free_spans[lower_idx].1 += count;
            },
            (None, Some(upper_idx)) => {
                self.free_spans[upper_idx].0 = start;
                self.free_spans[upper_idx].1 += count;
            },
            (None, None) => {
                self.free_spans.push((start, count));
            },
        }
    }
}

#[derive(Default)]
pub struct QuadBuffer(pub [FaceBuffer; 6]);

impl QuadBuffer {
    pub fn add(&mut self, quads: Vec<QuadData>, face: usize, chunk_pos: ChunkPos) {
        self.0[face].add(quads, chunk_pos);
    }

    pub fn remove(&mut self, chunk_pos: ChunkPos) {
        for face_buffer in self.0.iter_mut() {
            face_buffer.remove(chunk_pos);
        }
    }

    pub fn culled(&self, view_direction: Vec3, view_origin: Vec3) 
        -> (Vec<QuadData>, Vec<(usize, usize)>, Vec<ChunkBuffer>) 
    {
        let quad_count = self.0.iter().map(|face_buffer| face_buffer.quads.len()).sum();
        let mut quad_buffer = Vec::with_capacity(quad_count);
        let chunk_count = self.0.iter().map(|face_buffer| face_buffer.chunks.len()).sum();
        let mut chunk_buffer = Vec::with_capacity(chunk_count);
        let mut indirect_buffer = Vec::with_capacity(chunk_count);
        // No culling for now
        for (face_index, face_buffer) in self.0.iter().enumerate() {
            let offset = quad_buffer.len();
            let chunk_offset = chunk_buffer.len();
            quad_buffer.extend_from_slice(&face_buffer.quads);
            chunk_buffer.extend_from_slice(
                &face_buffer.chunk_spans.iter().map(|(chunk_pos, _, _)| ChunkBuffer {
                    x: chunk_pos.x,
                    y: chunk_pos.y,
                    z: chunk_pos.z,
                    face: face_index as u32,
                }).collect::<Vec<_>>()
            );
            indirect_buffer.extend_from_slice(
                &face_buffer.chunk_spans.iter()
                    .map(|(_, i, count)| (*i+offset, *count))
                    .collect::<Vec<_>>()
            );
            // Manually set draw_id for each quad 
            // (ugly workaround that will be removed when wgpu supports proper draw id)
            for (span_idx, (_, start, count)) in face_buffer.chunk_spans.iter().enumerate() {
                let start = offset + *start;
                let draw_id = (chunk_offset + span_idx) as u32;
                for i in start..(start + *count) {
                    quad_buffer[i].draw_id = draw_id;
                }
            }
        }
        (quad_buffer, indirect_buffer, chunk_buffer)
    }
}

#[derive(Clone, Component, Default, Deref)]
pub struct InstanceQuads(pub Arc<RwLock<QuadBuffer>>);

#[cfg(test)]
mod tests {
    use crate::{render::quad_data::{FaceBuffer, QuadData}, world::{ChunkPos, Realm}};

    fn random_quads() -> Vec<QuadData> {
        use rand::Rng;
        let mut rng = rand::rng();
        (0..rng.random_range(10..1000)).map(|_| {
            QuadData {
                quad_pos: rng.random(),
                quad_info: 0,
                draw_id: 0,
            }
        }).collect()
    }

    /// Produces a face buffer that has had a good amount of random additions and removal
    fn random_face_buffer() -> FaceBuffer {
        use rand::seq::IndexedRandom;

        let mut rng = rand::rng();
        let mut face_buffer = FaceBuffer::default();
        for i in 0..1000 {
            // Add two chunks and remove one to try to induce fragmentation
            let quads = random_quads();
            let chunk_pos = ChunkPos { realm: Realm::Overworld, x: i, y: 0, z: rand::random() };
            face_buffer.add(quads, chunk_pos);
            let quads = random_quads();
            let chunk_pos = ChunkPos { realm: Realm::Overworld, x: i, y: 1, z: rand::random() };
            face_buffer.add(quads, chunk_pos);
            let (random_chunk_pos, _, _) = face_buffer.chunk_spans.choose(&mut rng).unwrap();
            face_buffer.remove(*random_chunk_pos);
        }
        face_buffer
    }

    #[test]
    fn test_free_span_merging() {
        let mut face_buffer = random_face_buffer();
        // Remove all the chunks
        let remaining_chunks = face_buffer.chunks.clone();
        for chunk_pos in remaining_chunks {
            face_buffer.remove(chunk_pos);
        }
        // There should be a single free span covering the entire buffer
        assert_eq!(face_buffer.free_spans.len(), 1);
        assert_eq!(face_buffer.free_spans[0].0, 0);
        assert_eq!(face_buffer.free_spans[0].1, face_buffer.quads.len());
    }

    #[test]
    fn test_chunk_spans() {
        let face_buffer = random_face_buffer();
        // check that chunk spans + free spans cover the entire buffer without overlap
        let mut covered = vec![false; face_buffer.quads.len()];
        for (start, count) in face_buffer.free_spans.iter() {
            for i in *start..(*start + *count) {
                assert!(!covered[i], "Overlap in free spans");
                covered[i] = true;
            }
        }
        for (_, start, count) in face_buffer.chunk_spans.iter() {
            for i in *start..(*start + *count) {
                assert!(!covered[i], "Overlap of a chunk span with a free span or a previous chunk span");
                covered[i] = true;
            }
        }
        assert!(covered.iter().all(|&c| c), "Not all quads are covered by spans");
    }
}