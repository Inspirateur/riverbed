use crate::blocs::{pos2d::chunks_in_col, ChunkPos, ColPos, TrackedChunk};
use bevy::prelude::*;
use dashmap::DashMap;
use itertools::iproduct;
use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Component, Clone, Copy)]
pub struct RenderDistance(pub u32);

#[derive(Resource, Clone)]
pub struct LoadArea {
    pub center: ColPos,
    pub col_dists: HashMap<ColPos, u32>,
    dist: i32,
}

pub fn range_around(a: i32, dist: i32) -> RangeInclusive<i32> {
    (a-dist)..=(a+dist)
}

impl LoadArea {
    pub fn new(center: ColPos, render_dist: RenderDistance) -> Self {
        let dist = render_dist.0 as i32;
        Self {
            center: center,
            col_dists: iproduct!(
                range_around(center.x, dist),
                range_around(center.z, dist)
            ).map(|(x, z)| (
                ColPos {
                    x, z, realm: center.realm
                }, x.abs_diff(center.x).max(z.abs_diff(center.z))
            )).collect(),
            dist
        }
    }

    pub fn empty() -> Self {
        Self {
            center: ColPos::default(),
            col_dists: HashMap::new(),
            dist: 0,
        }
    }

    fn col_from_delta(&self, dx: i32, dz: i32) -> ColPos {
        ColPos {
            x: self.center.x + dx,
            z: self.center.z + dz,
            realm: self.center.realm
        }
    }

    fn first_change_in(&self, col_pos: ColPos, chunks: &DashMap<ChunkPos, TrackedChunk>) -> Option<ChunkPos> {
        for chunk_pos in chunks_in_col(&col_pos) {
            let Some(chunk) = chunks.get(&chunk_pos) else {
                continue;
            };
            if chunk.changed {
                return Some(chunk_pos);
            }
        }
        None
    }

    fn closest_change(&self, chunks: &DashMap<ChunkPos, TrackedChunk>) -> Option<ChunkPos> {
        if let Some(chunk_pos) = self.first_change_in(self.center, chunks) {
            return Some(chunk_pos);
        }
        for d in 1..=self.dist {
            for dx in -d..=d {
                if let Some(chunk_pos) = self.first_change_in(self.col_from_delta(dx, d), chunks) {
                    return Some(chunk_pos);
                }
                if let Some(chunk_pos) = self.first_change_in(self.col_from_delta(dx, -d), chunks) {
                    return Some(chunk_pos);
                }
            }
            for dz in (-d+1)..d {
                if let Some(chunk_pos) = self.first_change_in(self.col_from_delta(d, dz), chunks) {
                    return Some(chunk_pos);
                }
                if let Some(chunk_pos) = self.first_change_in(self.col_from_delta(-d, dz), chunks) {
                    return Some(chunk_pos);
                }
            }
        }
        None
    }

    pub fn pop_closest_change(&self, chunks: &DashMap<ChunkPos, TrackedChunk>) -> Option<(ChunkPos, u32)> {
        let span = info_span!("selecting chunk to mesh", name = "selecting chunk to mesh").entered();
        let res = self.closest_change(chunks)?;
        span.exit();
        let Some(mut chunk) = chunks.get_mut(&res) else {
            println!("couldn't get_mut chunk {:?}", res);
            return None
        };
        chunk.changed = false;
        Some((res, res.x.abs_diff(self.center.x).max(res.z.abs_diff(self.center.z))))
    }
}