use crate::{chunk::CHUNK_S1, pos::Pos, realm::Realm, col_commands::ColCommands};
use bevy::prelude::Query;
use bevy::prelude::*;
use itertools::iproduct;
use std::ops::{Deref, Sub};

#[derive(Component, Clone, Copy)]
pub struct LoadArea {
    pub realm: Realm,
    pub col: (i32, i32),
    pub dist: u32,
}

impl LoadArea {
    pub fn contains(&self, chunk_x: i32, chunk_z: i32) -> bool {
        // checks if a chunk is in this Player loaded area (assuming they're in the same realm)
        i32::max((self.col.0 - chunk_x).abs(), (self.col.1 - chunk_z).abs()) < self.dist as i32
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32)> {
        let dist = self.dist as i32;
        iproduct!(
            (self.col.0 - dist)..=(self.col.0 + dist),
            (self.col.1 - dist)..=(self.col.1 + dist)
        )
    }
}

impl Sub<LoadArea> for LoadArea {
    type Output = Vec<(i32, i32)>;

    fn sub(self, rhs: LoadArea) -> Self::Output {
        if self.realm != rhs.realm {
            self.iter().collect()
        } else {
            self.iter()
                .filter(|(cx, cz)| !rhs.contains(*cx, *cz))
                .collect()
        }
    }
}

pub fn update_load_area(mut query: Query<(&Pos, &mut LoadArea), Changed<Pos>>) {
    for (pos, mut load_area) in query.iter_mut() {
        let col = (
            (*pos).x as i32 / CHUNK_S1 as i32,
            (*pos).z as i32 / CHUNK_S1 as i32,
        );
        // we're checking before modifying to avoid triggering Change detection
        if pos.realm != load_area.realm {
            load_area.realm = pos.realm;
        }
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

pub fn load_order(
    mut commands: Commands,
    mut query: Query<(&LoadArea, Option<&mut LoadAreaOld>, Entity), Changed<LoadArea>>,
    mut world: ResMut<ColCommands>,
) {
    for (load_area, load_area_old_opt, entity) in query.iter_mut() {
        // compute the columns to load and unload & update old load area
        let new_load_area_old = LoadAreaOld(load_area.clone());
        if let Some(mut load_area_old) = load_area_old_opt {
            let mut load_area_ext = load_area.clone();
            load_area_ext.dist += 1;
            world.load(*load_area - **load_area_old, load_area.realm, entity.id());
            world.unload(
                **load_area_old - load_area_ext,
                load_area_old.realm,
                entity.id(),
            );
            *load_area_old = new_load_area_old;
        } else {
            commands.entity(entity).insert(new_load_area_old);
            world.load(load_area.iter().collect(), load_area.realm, entity.id());
        }
    }
}
