use serde::Deserialize;

use crate::blocks::Block;

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
    IronPickaxe,
    IronAxe,
    IronShovel,    
    #[serde(untagged)]
    Block(Block),
}

pub struct Efficiency(pub f32);

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
