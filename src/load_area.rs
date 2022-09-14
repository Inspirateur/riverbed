use crate::{bloc_pos::ChunkPos2D, col_commands::ColCommands, pos::Pos};
use bevy::prelude::Query;
use bevy::prelude::*;
use itertools::iproduct;
use std::ops::{Deref, Sub};

#[derive(Component, Clone, Copy)]
pub struct LoadArea {
    pub col: ChunkPos2D,
    pub dist: u32,
}

impl LoadArea {
    pub fn contains(&self, col: ChunkPos2D) -> bool {
        // checks if a chunk is in this Player loaded area (assuming they're in the same realm)
        self.col.dist(col) < self.dist as i32
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32)> {
        let dist = self.dist as i32;
        iproduct!(
            (self.col.x - dist)..=(self.col.x + dist),
            (self.col.z - dist)..=(self.col.z + dist)
        )
    }
}

impl Sub<LoadArea> for LoadArea {
    type Output = Vec<(i32, i32)>;

    fn sub(self, rhs: LoadArea) -> Self::Output {
        if self.col.realm != rhs.col.realm {
            self.iter().collect()
        } else {
            self.iter()
                .filter(|(x, z)| {
                    !rhs.contains(ChunkPos2D {
                        realm: self.col.realm,
                        x: *x,
                        z: *z,
                    })
                })
                .collect()
        }
    }
}

pub fn update_load_area(mut query: Query<(&Pos, &mut LoadArea), Changed<Pos>>) {
    for (pos, mut load_area) in query.iter_mut() {
        let col = ChunkPos2D::from(*pos);
        // we're checking before modifying to avoid triggering unnecessary Change detection
        if col != load_area.col {
            load_area.col = col;
        }
    }
}

#[derive(Component)]
pub struct LoadAreaOld(LoadArea);

impl Deref for LoadAreaOld {
    type Target = LoadArea;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// FIXME: COLUMN LOADING IS MEGA BUGGED, there's random holes and out of range cols, figure out why 
pub fn load_order(
    mut commands: Commands,
    mut query: Query<(&LoadArea, Option<&mut LoadAreaOld>, Entity), Changed<LoadArea>>,
    mut world: ResMut<ColCommands>,
) {
    for (load_area, load_area_old_opt, entity) in query.iter_mut() {
        // compute the columns to load and unload & update old load area
        let load_area_clone = LoadAreaOld(load_area.clone());
        if let Some(mut load_area_old) = load_area_old_opt {
            let mut load_area_ext = load_area.clone();
            load_area_ext.dist += 1;
            world.load(
                *load_area - **load_area_old,
                load_area.col.realm,
                entity.id(),
            );
            world.unload(
                **load_area_old - load_area_ext,
                load_area_old.col.realm,
                entity.id(),
            );
            *load_area_old = load_area_clone;
        } else {
            commands.entity(entity).insert(load_area_clone);
            world.load(load_area.iter().collect(), load_area.col.realm, entity.id());
        }
    }
}
