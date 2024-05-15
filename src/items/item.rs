use serde::Deserialize;

use crate::blocks::Block;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ToolKind {
    Default,
    Item(Item),
    ToolFamily(ToolFamily)
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ToolFamily {
    Pickaxe,
    Axe,
    Shovel,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Item {
    Block(Block),
    Lime,
    Stick,
    Rock,
    IronPickaxe,
    IronAxe,
    IronShovel,
}

pub struct Efficiency(f32);

impl Item {
    pub fn tool_family(&self) -> Option<(ToolFamily, Efficiency)> {
        match self {
            Item::IronPickaxe => Some((ToolFamily::Pickaxe, Efficiency(2.))),
            Item::IronAxe => Some((ToolFamily::Axe, Efficiency(2.))),
            Item::IronShovel => Some((ToolFamily::Shovel, Efficiency(2.))),
            _ => None
        }
    }
}