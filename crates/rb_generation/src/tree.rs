use crate::trees::*;
use rb_world::{BlockPos, StructureTrait, VoxelWorld};
use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, EnumString)]
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
    Bamboo,
}

pub struct TreeSeed {
    pub species: Tree,
    pub size: f32,
    pub pos: BlockPos,
}

impl StructureTrait for TreeSeed {
    fn origin(&self) -> BlockPos {
        self.pos
    }

    fn grow(&self, world: &VoxelWorld, seed: i32) {
        if !world.get_block_safe(self.pos).is_fertile_soil() {
            return;
        }
        match self.species {
            Tree::Spruce => grow_spruce(world, self.pos, seed, self.size),
            Tree::Birch => grow_birch(world, self.pos, seed, self.size),
            Tree::Cypress => grow_cypress(world, self.pos, seed, self.size),
            Tree::Oak | Tree::Chestnut | Tree::Ironwood => {
                grow_oak(world, self.pos, seed, self.size)
            }
            Tree::Acacia => grow_acacia(world, self.pos, seed, self.size),
            Tree::Sequoia => grow_sequoia(world, self.pos, seed, self.size),
            Tree::Palm | Tree::Baobab => grow_baobab(world, self.pos, seed, self.size),
            _ => {}
        }
    }

    fn max_block_width(&self) -> i32 {
        30
    }
}
