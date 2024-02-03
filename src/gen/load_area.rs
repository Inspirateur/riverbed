use crate::blocs::ColPos;
use bevy::prelude::*;
use itertools::iproduct;
use std::collections::HashMap;

#[derive(Component, Clone, Copy)]
pub struct RenderDistance(pub u32);

#[derive(Component)]
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
}
