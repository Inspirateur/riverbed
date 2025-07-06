use crate::{world::{ChunkPos, ColPos, TrackedChunk}, RENDER_DISTANCE};
use bevy::prelude::*;
use dashmap::DashMap;
use itertools::iproduct;
use std::ops::RangeInclusive;

pub struct PlayerAreaDiff {
    pub exclusive_in_self: Vec<ColPos>,
    pub exclusive_in_other: Vec<ColPos>,
}

pub fn range_around(a: i32, dist: i32) -> RangeInclusive<i32> {
    (a - dist)..=(a + dist)
}

impl ColPos {
    fn in_rd(&self, other: &ColPos) -> bool {
        (self.x - other.x).abs() <= RENDER_DISTANCE as i32 &&
        (self.z - other.z).abs() <= RENDER_DISTANCE as i32 &&
        self.realm == other.realm
    }

    fn rd_area(&self) -> impl Iterator<Item = ColPos> {
        iproduct!(
            range_around(self.x, RENDER_DISTANCE as i32),
            range_around(self.z, RENDER_DISTANCE as i32)
        ).map(|(x, z)| ColPos { x, z, realm: self.realm })
    }

    // Given RENDER_DISTANCE and another column position, returns the columns that are in self area but not in the other area,
    // and the columns that are in the other area but not in self area.
    pub fn player_area_diff(&self, other: Option<ColPos>) -> PlayerAreaDiff {
        let exclusive_in_self = if let Some(other_col) = other {
            self.rd_area().filter(|col| !col.in_rd(&other_col)).collect()
        } else {
            self.rd_area().collect()
        };

        let exclusive_in_other = if let Some(other_col) = other {
            other_col.rd_area().filter(|col| !col.in_rd(self)).collect()
        } else {
            Vec::new()
        };

        PlayerAreaDiff {
            exclusive_in_self, exclusive_in_other,
        }
    }

    fn closest_change(&self, chunks: &DashMap<ChunkPos, TrackedChunk>) -> Option<ChunkPos> {
        chunks
            .iter()
            .filter_map(|entry| {
                if entry.value().changed {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .min_by_key(|chunk_pos| <ColPos>::from(*chunk_pos).dist(*self))
    }

    pub fn pop_closest_change(
        &self,
        chunks: &DashMap<ChunkPos, TrackedChunk>,
    ) -> Option<(ChunkPos, u32)> {
        let span =
            info_span!("selecting chunk to mesh", name = "selecting chunk to mesh").entered();
        let res = self.closest_change(chunks)?;
        span.exit();
        let Some(mut chunk) = chunks.get_mut(&res) else {
            println!("couldn't get_mut chunk {:?}", res);
            return None;
        };
        chunk.changed = false;
        Some((
            res,
            res.x
                .abs_diff(self.x)
                .max(res.z.abs_diff(self.z)),
        ))
    }
}
