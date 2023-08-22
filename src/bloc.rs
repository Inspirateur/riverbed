use std::ops::Range;
use block_mesh::{VoxelVisibility, Voxel, MergeVoxel};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

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

pub type Soils = Vec<([Range<f32>; 2], Bloc)>;

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