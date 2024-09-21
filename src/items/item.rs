use std::str::FromStr;

use serde::Deserialize;
use crate::Block;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ToolKind {
    Default,
    #[serde(untagged)]
    Item(Item),
    #[serde(untagged)]
    ToolFamily(ToolFamily)
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ToolFamily {
    Pickaxe,
    Axe,
    Shovel,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Item {
    Lime,
    Stick,
    Rock,
    StoneAxe,
    IronIngot,
    IronPickaxe,
    IronAxe,
    IronShovel,
    #[serde(untagged)]
    Block(Block),
}

impl FromStr for Item {
    type Err = json5::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: We'd like to implement FromStr for Item using strum but it doesn't seem to support nested enums
        json5::from_str(&format!("'{}'", s))
    }
}

pub struct Efficiency(pub f32);

impl Item {
    pub fn tool_family(&self) -> Option<(ToolFamily, Efficiency)> {
        match self {
            Item::StoneAxe => Some((ToolFamily::Axe, Efficiency(1.))),
            Item::IronPickaxe => Some((ToolFamily::Pickaxe, Efficiency(2.))),
            Item::IronAxe => Some((ToolFamily::Axe, Efficiency(2.))),
            Item::IronShovel => Some((ToolFamily::Shovel, Efficiency(2.))),
            _ => None
        }
    }
}
