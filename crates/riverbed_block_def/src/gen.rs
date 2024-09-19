use std::collections::{BTreeSet, HashSet};

use itertools::Itertools;

use crate::parse::{BlockFrag, IR};

pub fn generate(ir: &IR) -> String {
    let mut blocks: BTreeSet<String> = BTreeSet::new();
    for block_pattern in ir.decl.iter() {
        let families = block_pattern.0.iter().filter_map(|frag| match frag { 
            BlockFrag::Ident(_) => None,
            BlockFrag::SetName(set_name) => Some(set_name) 
        }).collect::<HashSet<_>>();
        for frags in block_pattern.0.iter()
            .map(|frag| match frag {
                BlockFrag::Ident(ident) => vec![ident],
                BlockFrag::SetName(set_name) => ir.sets.get(set_name).unwrap().iter().collect()
            }).multi_cartesian_product() 
        {
            blocks.insert(frags.into_iter().map(|s| s.as_str()).collect());
        }
    }
    format!("pub enum Blocks {{\n\t{}\n}}", blocks.into_iter().join(",\n\t"))
}