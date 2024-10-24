use std::collections::HashMap;
use bevy::prelude::{Component, Resource};
use serde::Deserialize;
use crate::{ui::ItemHolder, BlockFamily};

use super::{Item, Stack};


#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct LitFurnace {
    pub firing_sec: f32,
    pub fuel_sec: f32,
    pub output: Item,
}

#[derive(Debug, Deserialize)]
pub struct FiringValue {
    pub min_temp: u32,
    pub smelt_time: u32,
    pub output: Item,
}

#[derive(Debug, Resource, Deserialize)]
pub struct FiringTable(HashMap<Item, FiringValue>);

impl FiringTable {
    pub fn get(&self, furnace_content: &ItemHolder, furnace_temp: u32) -> Option<LitFurnace> {
        let ItemHolder::Furnace { fuel, material, output } = furnace_content else {
            return None;
        };
        let Stack::Some(material_item, _) = material else {
            return None;
        };
        let fuel_time = match fuel {
            Stack::Some(item, _) => match item {
                Item::Coal => 20.,
                Item::Block(block) => {
                    let families = block.families();
                    if families.contains(&BlockFamily::Log) {
                        10.
                    } else if families.contains(&BlockFamily::Planks) {
                        2.
                    } else {
                        0.
                    }
                }
                _ => 0.
            },
            Stack::None => 0.,
        }*1000./furnace_temp as f32;
        // Fuel is not suitable
        if fuel_time == 0. {
            return None;
        };
        let Some(firing_value) = self.0.get(material_item) else {
            return None;
        };
        if furnace_temp < firing_value.min_temp {
            return None;
        }
        if !output.can_add(Stack::Some(firing_value.output, 1)) {
            return None;
        }
        Some(LitFurnace {
            firing_sec: firing_value.smelt_time as f32*firing_value.min_temp as f32/furnace_temp as f32,
            fuel_sec: fuel_time,
            output: firing_value.output,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_table() {
        let config = r#"
        {
            Clay: { min_temp: 600, smelt_time: 5, output: "Brick" },
            IronOre: { min_temp: 1500, smelt_time: 10, output: "IronIngot" },
        }
        "#;
        let firing_table: FiringTable = json5::from_str(config).unwrap();
        println!("{:?}", firing_table);
    }
}
