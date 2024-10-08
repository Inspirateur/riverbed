use std::collections::HashMap;

use serde::Deserialize;

use super::Item;

#[derive(Debug, Deserialize)]
struct FiringValue {
    min_temp: u32,
    smelt_time: u32,
    output: Item,
}

#[derive(Debug, Deserialize)]
pub struct FiringTable(HashMap<Item, FiringValue>);

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
