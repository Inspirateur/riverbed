use bevy::prelude::*;
use itertools::iproduct;
use std::ops::RangeInclusive;

use crate::world::pos::ColPos;


pub struct PlayerAreaDiff {
    pub exclusive_in_self: Vec<ColPos>,
    pub exclusive_in_other: Vec<ColPos>,
}

pub fn range_around(a: i32, dist: i32) -> RangeInclusive<i32> {
    (a - dist)..=(a + dist)
}

impl ColPos {
    fn in_rd(&self, other: &ColPos, render_distance: f32) -> bool {
        (self.x - other.x).abs() <= render_distance as i32 &&
        (self.z - other.z).abs() <= render_distance as i32 &&
        self.realm == other.realm
    }

    fn rd_area(&self, render_distance: f32) -> impl Iterator<Item = ColPos> {
        let realm = self.realm;
        iproduct!(
            range_around(self.x, render_distance as i32),
            range_around(self.z, render_distance as i32)
        ).map(move |(x, z)| ColPos { x, z, realm })
    }

    // Given RENDER_DISTANCE and another column position, returns the columns that are in self area but not in the other area,
    // and the columns that are in the other area but not in self area.
    pub fn player_area_diff(&self, other: Option<ColPos>, render_distance: f32) -> PlayerAreaDiff {
        let exclusive_in_self = if let Some(other_col) = other {
            self.rd_area(render_distance).filter(|col| !col.in_rd(&other_col, render_distance)).collect()
        } else {
            self.rd_area(render_distance).collect()
        };

        let exclusive_in_other = if let Some(other_col) = other {
            other_col.rd_area(render_distance).filter(|col| !col.in_rd(self, render_distance)).collect()
        } else {
            Vec::new()
        };

        PlayerAreaDiff {
            exclusive_in_self, exclusive_in_other,
        }
    }
}
