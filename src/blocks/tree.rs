use std::{ops::Range, str::FromStr};
use ron::de::SpannedError;
use serde::Deserialize;

use super::{growables::*, BlockPos, Blocks};


#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
pub enum Tree {
    Oak,
    Spruce,
    Sequoia,
    Palm,
    Birch,
    Chestnut,
    Cypress,
    Ironwood,
    Baobab,
    Cactus,
    Acacia,
    Bamboo
}

impl Tree {
    pub fn grow(&self, world: &Blocks, pos: BlockPos, seed: i32, dist: f32) {
        if !world.get_block_safe(pos).is_fertile_soil() { return; }
        match self {
            Tree::Spruce => grow_spruce(world, pos, seed, dist),
            Tree::Birch => grow_birch(world, pos, seed, dist),
            Tree::Cypress => grow_cypress(world, pos, seed, dist),
            Tree::Oak | Tree::Chestnut | Tree::Ironwood => grow_oak(world, pos, seed, dist),
            Tree::Acacia => grow_acacia(world, pos, seed, dist),
            Tree::Sequoia => grow_sequoia(world, pos, seed, dist),
            Tree::Palm | Tree::Baobab => grow_baobab(world, pos, seed, dist),
            _ => {}
        }
    }
}

impl FromStr for Tree {
    type Err = SpannedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::from_str(s)
    }
}

pub type Trees = Vec<([Range<f32>; 4], Tree)>;
