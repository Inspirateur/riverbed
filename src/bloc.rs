use std::ops::Range;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, EnumString, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Bloc {
    #[default]
    Air,
    Dirt,
    Grass,
    Stone,
    OakWood,
    OakLeave,
    Sand,
    Ice,
    Snow,
    Mud,
    Bedrock,
}

pub type Soils = Vec<([Range<f32>; 2], Bloc)>;
