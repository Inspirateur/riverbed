use std::collections::HashMap;
use serde::Deserialize;
use crate::blocks::{Block, BlockFamily};
use super::item::{Item, ToolKind};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
enum DropKind {
    #[serde(rename = "Self")]
    Itself,
    #[serde(untagged)]
    Item(Item),
}

impl Into<Item> for (DropKind, Block) {
    fn into(self) -> Item {
        match self.0 {
            DropKind::Itself => Item::Block(self.1),
            DropKind::Item(item) => item,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(untagged)]
pub enum BlockKind {
    Block(Block),
    Family(BlockFamily),
}

pub enum DropQuantity {
    Fixed(u32),
    Range { min: u32, max: u32 }
}

#[derive(Default, Debug, Deserialize)]
struct LootEntryPartial {
    pub hardness: Option<f32>,
    pub drops: Option<DropKind>,
    pub min: Option<u32>,
    pub max: Option<u32>
}

impl LootEntryPartial {
    fn complete_with(&mut self, other: &LootEntryPartial, efficiency: f32) {
        if self.hardness.is_none() {
            self.hardness = other.hardness.map(|h| h/efficiency);
        }
        if self.drops.is_none() {
            self.drops = other.drops;
            self.min = other.min;
            self.max = other.max;
        }
    }

    fn is_complete(&self) -> bool {
        self.hardness.is_some() && self.drops.is_some()
    }

    fn quantity(&self) -> DropQuantity {
        if let Some(min) = self.min {
            DropQuantity::Range { min, max: self.max.unwrap() }
        } else {
            DropQuantity::Fixed(1)
        }
    }
}

pub struct LootEntry {
    pub hardness: Option<f32>,
    pub drops: Option<Item>,
    pub quantity: Option<DropQuantity>,
}

impl From<(LootEntryPartial, Block)> for LootEntry {
    fn from((entry, block): (LootEntryPartial, Block)) -> Self {
        Self {
            hardness: entry.hardness,
            quantity: if entry.drops.is_none() { None } else { Some(entry.quantity()) },
            drops: entry.drops.map(|drop| (drop, block).into()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockLootTable(HashMap<ToolKind, HashMap<BlockKind, LootEntryPartial>>);

impl BlockLootTable {
    fn try_to_complete(&self, partial_entry: &mut LootEntryPartial, tool_kind: &ToolKind, block_kind: &BlockKind, efficiency: f32) {
        if let Some(loot_entries) = self.0.get(tool_kind) {
            if let Some(loot_entry) = loot_entries.get(block_kind) {
                partial_entry.complete_with(&loot_entry, efficiency)
            }
        }
    }

    pub fn get(&self, tool_opt: Option<&Item>, block: &Block) -> LootEntry {
        let mut partial_entry = LootEntryPartial::default();
        if let Some(tool) = tool_opt {
            // Check exact tool and exact block
            self.try_to_complete(&mut partial_entry, &ToolKind::Item(*tool), &BlockKind::Block(*block), 1.);
            if partial_entry.is_complete() { return (partial_entry, *block).into(); }
            // Check exact tool and block family
            for block_family in block.families() {
                self.try_to_complete(&mut partial_entry, &ToolKind::Item(*tool), &BlockKind::Family(block_family), 1.);
                if partial_entry.is_complete() { return (partial_entry, *block).into(); }
            }
            // Check tool family and exact block
            if let Some((tool_family, efficiency)) = tool.tool_family() {
                self.try_to_complete(&mut partial_entry, &ToolKind::ToolFamily(tool_family), &BlockKind::Block(*block), efficiency.0);
            }
            if partial_entry.is_complete() { return (partial_entry, *block).into(); }
            // Check tool family and block family
            if let Some((tool_family, efficiency)) = tool.tool_family() {
                for block_family in block.families() {
                    self.try_to_complete(&mut partial_entry, &ToolKind::ToolFamily(tool_family), &BlockKind::Family(block_family), efficiency.0);
                    if partial_entry.is_complete() { return (partial_entry, *block).into(); }
                }
            }
        }
        // Check default and exact block
        self.try_to_complete(&mut partial_entry, &ToolKind::Default, &BlockKind::Block(*block), 1.);
        if partial_entry.is_complete() { return (partial_entry, *block).into(); }
        // Check default and block family
        for block_family in block.families() {
            self.try_to_complete(&mut partial_entry, &ToolKind::Default, &BlockKind::Family(block_family), 1.);
            if partial_entry.is_complete() { return (partial_entry, *block).into(); }
        }
        return (partial_entry, *block).into();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_fallback() {
        let config = r#"
        { 
            Default: {
                Stone: { hardness: 7, drops: "Rock", min: 4, max: 6 }, 
                Limestone: { hardness: 5.5, drops: "Lime", min: 1, max: 2 },
            },
            Pickaxe: { 
                Stone: { hardness: 2.5, drops: "Self" }, 
            },
            IronPickaxe: { 
                Stone: { hardness: 1. } 
            }
        }
        "#;
        let block_looting: BlockLootTable = json5::from_str(config).unwrap();
        println!("{:?}", block_looting);
        assert_eq!(block_looting.get(Some(&Item::IronPickaxe), &Block::Limestone).drops, Some(Item::Block(Block::Limestone)));
        assert_eq!(block_looting.get(Some(&Item::IronPickaxe), &Block::Limestone).hardness, Some(1.));
        assert_eq!(block_looting.get(Some(&Item::Stick), &Block::Limestone).drops, Some(Item::Lime));
        assert_eq!(block_looting.get(Some(&Item::Stick), &Block::Cobblestone).drops, Some(Item::Rock));
        assert_eq!(block_looting.get(None, &Block::Cobblestone).drops, Some(Item::Rock));
    }
}
