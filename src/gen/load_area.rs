use crate::blocks::{ChunkPos, ColPos, TrackedChunk};
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
        }
    }

    pub fn empty() -> Self {
        Self {
            center: ColPos::default(),
            col_dists: HashMap::new(),
        }
    }

    fn closest_change(&self, chunks: &DashMap<ChunkPos, TrackedChunk>) -> Option<ChunkPos> {
        chunks.iter().filter_map(|entry| if entry.value().changed {
            Some(entry.key().clone())
        } else {
            None
        }).min_by_key(|chunk_pos| <ColPos>::from(*chunk_pos).dist(self.center))
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