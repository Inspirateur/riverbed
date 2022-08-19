use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum Bloc {
    Air,
    Dirt,
    Grass,
    Stone,
    OakWood,
    OakLeave,
    Sand,
    Ice,
}
