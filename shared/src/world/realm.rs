use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash, Component, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Realm {
    #[default]
    Overworld,
    Aether,
    Nether
}