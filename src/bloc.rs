use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, EnumString, Hash)]
#[strum(ascii_case_insensitive)]
pub enum Bloc {
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
}
