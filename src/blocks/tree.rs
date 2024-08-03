use std::ops::Range;
use serde::Deserialize;
use strum_macros::EnumString;
use super::growables::*;
use crate::world::{BlockPos, VoxelWorld};


#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
#[derive(EnumString)]
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
    pub fn grow(&self, world: &VoxelWorld, pos: BlockPos, seed: i32, dist: f32) {
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

pub type Trees = Vec<([Range<f32>; 4], Tree)>;
