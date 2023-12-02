use std::ops::Range;
use block_mesh::{VoxelVisibility, Voxel, MergeVoxel};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumIter};
use super::{Blocs, BlocPos, growables::*};

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, EnumString, EnumIter, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Bloc {
    #[default]
    Air,
    Dirt,
    GrassBlock,
    Glass,
    Stone,
    BirchLog,
    BirchLeaves,
    OakLog,
    OakLeaves,
    SpruceLog,
    SpruceLeaves,
    SequoiaLog,
    SequoiaLeaves,
    Sand,
    Ice,
    Snow,
    Mud,
    Bedrock,
}


impl Bloc {
    pub fn friction(&self) -> f32 {
        match self {
            Bloc::Air => 0.2,
            Bloc::Ice => 0.3,
            _ => 1.
        }
    }

    pub fn slowing(&self) -> f32 {
        match self {
            Bloc::Mud => 0.8,
            _ => 1.
        }
    }

    pub fn traversable(&self) -> bool {
        match self {
            Bloc::Air => true,
            _ => false,
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

impl Voxel for Bloc {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Bloc::Air => VoxelVisibility::Empty,
            Bloc::Glass | Bloc::OakLeaves | Bloc::SpruceLeaves => VoxelVisibility::Translucent,
            _ => VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for Bloc {
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
    pub fn grow(&self, world: &mut Blocs, pos: BlocPos, seed: i32, dist: f32) {
        match self {
            Tree::Spruce => grow_spruce(world, pos, seed, dist),
            Tree::Birch => grow_birch(world, pos, seed, dist),
            Tree::Cypress => grow_cypress(world, pos, seed, dist),
            Tree::Oak | Tree::Chestnut | Tree::Ironwood | Tree::Acacia => grow_oak(world, pos, seed, dist),
            Tree::Sequoia | Tree::Palm | Tree::Baobab => grow_sequoia(world, pos, seed, dist),
            _ => {}
        }
    }
}

pub type Soils = Vec<([Range<f32>; 2], Bloc)>;
pub type Trees = Vec<([Range<f32>; 4], Tree)>;
