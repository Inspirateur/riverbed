use std::collections::HashMap;
use serde::Deserialize;
use crate::blocks::{Block, BlockFamily};
use super::item::{Item, ToolKind};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DropKind {
    #[serde(rename = "Self")]
    Itself,
    Item(Item),
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum BlockKind {
    Block(Block),
    Family(BlockFamily),
}

pub enum DropQuantity {
    Fixed(u32),
    Range { min: u32, max: u32 }
}

#[derive(Debug, Deserialize)]
struct BreakEntry {
    hardness: Option<f32>,
    drops: Option<DropKind>,
    min: Option<u32>,
    max: Option<u32>
}


#[derive(Debug, Deserialize)]
struct BlockBreaking(HashMap<ToolKind, HashMap<BlockKind, BreakEntry>>);


#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn deser_block_breaking() {
        let fblock_breaking = fs::read_to_string("assets/data/block_breaking.ron").unwrap();
        let _block_breaking: BlockBreaking = ron::from_str(&fblock_breaking).unwrap();
    }
}
