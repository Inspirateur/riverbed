use crate::{world::ColPos, RENDER_DISTANCE};
use bevy::prelude::*;
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
}
