use std::ops::Range;
use block_mesh::{VoxelVisibility, Voxel, MergeVoxel};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::{Blocs, BlocPos, grow_oak};

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, EnumString, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Bloc {
    #[default]
    Air,
    Dirt,
    Grass,
    Glass,
    Stone,
    OakWood,
    OakLeave,
    Sand,
    Ice,
    Snow,
    Mud,
    Bedrock,
}

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
            Bloc::Glass | Bloc::OakLeave => VoxelVisibility::Translucent,
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
pub enum Plant {
    Oak,
    Spruce,
    Sequoia,
    Palm,
    Bush,
    Grass,
    Birch,
    Lavander,
    Lily,
    Chestnut,
    Cypress,
    Ironwood,
    Baobab,
    Cactus,
    Acacia,
    Bamboo
}

impl Plant {
    pub fn grow(&self, world: &mut Blocs, pos: BlocPos, dist: f32) {
        match self {
            _ => grow_oak(world, pos, dist)
        }
    }
}

pub type Soils = Vec<([Range<f32>; 2], Bloc)>;
pub type Plants = Vec<([Range<f32>; 5], Plant)>;
