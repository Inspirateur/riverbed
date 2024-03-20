use std::ops::Range;
use block_mesh::{VoxelVisibility, Voxel, MergeVoxel};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumIter};
use super::{Blocks, BlockPos, growables::*};

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, EnumString, EnumIter, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Block {
    #[default]
    Air,
    AcaciaLeaves,
    AcaciaLog,
    Bedrock,
    BirchLeaves,
    BirchLog,
    CoarseDirt,
    Cobblestone,
    Dirt,
    Endstone,
    GrassBlock,
    Glass,
    Ice,
    Mud,
    OakLeaves,
    OakLog,
    Podzol,
    Sand,
    SequoiaLeaves,
    SequoiaLog,
    SpruceLeaves,
    SpruceLog,
    Snow,
    SeaBlock,
    Stone,
}


impl Block {
    pub fn friction(&self) -> f32 {
        match self {
            Block::Air => 0.1,
            Block::Ice => 0.05,
            _ => 1.
        }
    }

    pub fn slowing(&self) -> f32 {
        match self {
            Block::Mud => 0.8,
            _ => 1.
        }
    }

    pub fn traversable(&self) -> bool {
        match self {
            Block::Air | Block::SeaBlock => true,
            _ => false,
        }
    }

    pub fn targetable(&self) -> bool {
        match self {
            Block::Air | Block::SeaBlock => false,
            _ => true
        }
    }

    pub fn is_soil(&self) -> bool {
        match self {
            Block::GrassBlock | Block::Podzol | Block::Snow
                => true,
            _ => false
        }
    }

    pub fn is_leaves(&self) -> bool {
        match self {
            Block::OakLeaves | Block::BirchLeaves | Block::SpruceLeaves | Block::SequoiaLeaves
                => true,
            _ => false
        }
    }

    pub fn is_transluscent(&self) -> bool {
        if self.is_leaves() {
            return true;
        }
        match self {
            Block::Glass | Block::SeaBlock => true,
            _ => false
        }
    }
}

#[derive(EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Face {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back
}

impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Block::Air => VoxelVisibility::Empty,
            block if block.is_transluscent() => VoxelVisibility::Translucent,
            _ => VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for Block {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString)]
#[strum(ascii_case_insensitive)]
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
        if !world.get_block_safe(pos).is_soil() { return; }
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

pub type Soils = Vec<([Range<f32>; 2], Block)>;
pub type Trees = Vec<([Range<f32>; 4], Tree)>;
