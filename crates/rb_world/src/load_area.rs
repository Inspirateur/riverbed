use crate::{ChunkPos2d, RENDER_DISTANCE};
use itertools::iproduct;
use std::ops::RangeInclusive;

pub struct PlayerAreaDiff {
    pub exclusive_in_self: Vec<ChunkPos2d>,
    pub exclusive_in_other: Vec<ChunkPos2d>,
}

pub fn range_around(a: i32, dist: i32) -> RangeInclusive<i32> {
    (a - dist)..=(a + dist)
}

fn in_rd(col: &ChunkPos2d, other: &ChunkPos2d) -> bool {
    (col.x - other.x).abs() <= RENDER_DISTANCE as i32
        && (col.z - other.z).abs() <= RENDER_DISTANCE as i32
        && col.realm == other.realm
}

pub fn rd_area(col: &ChunkPos2d) -> impl Iterator<Item = ChunkPos2d> + '_ {
    iproduct!(
        range_around(col.x, RENDER_DISTANCE as i32),
        range_around(col.z, RENDER_DISTANCE as i32)
    )
    .map(|(x, z)| ChunkPos2d {
        x,
        z,
        realm: col.realm,
    })
}

pub fn player_area_diff(col: &ChunkPos2d, other: Option<ChunkPos2d>) -> PlayerAreaDiff {
    let exclusive_in_self = if let Some(other_col) = other {
        rd_area(col).filter(|c| !in_rd(c, &other_col)).collect()
    } else {
        rd_area(col).collect()
    };

    let exclusive_in_other = if let Some(other_col) = other {
        rd_area(&other_col).filter(|c| !in_rd(c, col)).collect()
    } else {
        Vec::new()
    };

    PlayerAreaDiff {
        exclusive_in_self,
        exclusive_in_other,
    }
}
