use crate::blocs::{pos2d::chunks_in_col, ChunkPos, ColPos, TrackedChunk};
use bevy::prelude::*;
use dashmap::DashMap;
use itertools::iproduct;
use std::collections::HashMap;

#[derive(Component, Clone, Copy)]
pub struct RenderDistance(pub u32);

#[derive(Resource, Clone)]
pub struct LoadArea {
    pub center: ColPos,
    pub col_dists: HashMap<ColPos, u32>,
}

impl LoadArea {
    pub fn new(center: ColPos, render_dist: RenderDistance) -> Self {
        let dist = render_dist.0 as i32;
        Self {
            center: center,
            col_dists: iproduct!(
                (center.x - dist)..=(center.x + dist),
                (center.z - dist)..=(center.z + dist)
            ).map(|(x, z)| (
                ColPos {
                    x, z, realm: center.realm
                }, x.abs_diff(center.x).max(z.abs_diff(center.z))
            )).collect()
        }
    }

    pub fn empty() -> Self {
        Self {
            center: ColPos::default(),
            col_dists: HashMap::new()
        }
    }

    pub fn pop_closest_change(&self, chunks: &DashMap<ChunkPos, TrackedChunk>) -> Option<(ChunkPos, u32)> {
        let res = self.col_dists.iter()
            .flat_map(|(col_pos, dist)| 
                chunks_in_col(col_pos)
                .into_iter()
                .filter_map(|chunk_pos| {
                    let chunk = chunks.get(&chunk_pos)?;
                    if chunk.changed {
                        Some((chunk_pos, *dist))
                    } else {
                        None
                    }
                })
            )
            .min_by_key(|(_, dist)| *dist)?;
        let mut chunk = chunks.get_mut(&res.0)?;
        chunk.changed = false;
        Some(res)
    }
}