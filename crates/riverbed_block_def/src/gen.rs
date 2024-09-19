use std::collections::{BTreeMap, BTreeSet};
use itertools::Itertools;
use crate::parse::{BlockFrag, IR};

const BLOCK_FAM: &'static str = "BlockFamily";
const BLOCKS: &'static str = "Blocks";

pub fn generate_enum(name: &str, variants: &BTreeSet<String>) -> String {
    format!(
        "#[derive(Debug, PartialEq, EnumString, Eq, Serialize, Deserialize, Clone, Copy, Hash)]\npub enum {name} {{\n\t{}\n}}\n", 
        variants.into_iter().join(",\n\t")
    )
}

pub fn generate_family_impl(block_families: &BTreeMap<String, BTreeSet<&String>>) -> String {
    let match_arms = block_families.into_iter().map(
        |(block, families)| 
            format!("{BLOCKS}::{} => vec![{}]", block, families.into_iter().map(|f| format!("{BLOCK_FAM}::{f}")).join(", "))
        ).collect::<Vec<_>>();
    format!(
        "\tpub fn families(&self) -> Vec<{BLOCK_FAM}> {{\n\t\tmatch self {{\n\t\t\t{}\n\t\t}}\n\t}}", 
        match_arms.join(",\n\t\t\t")
    )
}

pub fn generate(ir: &IR) -> String {
    let mut blocks: BTreeSet<String> = BTreeSet::new();
    let mut block_families: BTreeMap<String, BTreeSet<&String>> = BTreeMap::new();
    for block_pattern in ir.decl.iter() {
        let families = block_pattern.0.iter().filter_map(|frag| match frag { 
            BlockFrag::Ident(_) => None,
            BlockFrag::SetName(set_name) => Some(set_name) 
        }).collect::<BTreeSet<_>>();
        for frags in block_pattern.0.iter()
            .map(|frag| match frag {
                BlockFrag::Ident(ident) => vec![ident],
                BlockFrag::SetName(set_name) => ir.sets.get(set_name).unwrap().iter().collect()
            }).multi_cartesian_product() 
        {
            let block: String = frags.into_iter().map(|s| s.as_str()).collect();
            block_families.insert(block.clone(), families.clone());
            blocks.insert(block);
        }
    }
    let mut code_blocks = Vec::new();
    code_blocks.push("use serde::{Deserialize, Serialize};".to_string());
    code_blocks.push("use strum_macros::EnumString;".to_string());
    code_blocks.push(String::new());
    code_blocks.push(generate_enum(BLOCK_FAM, &ir.sets.keys().map(|s| s.to_owned()).collect()));
    for (family, variants) in ir.sets.iter() {
        code_blocks.push(generate_enum(family, variants));
    }
    code_blocks.push(generate_enum(BLOCKS, &blocks));
    code_blocks.push(format!("impl {BLOCKS} {{"));
    code_blocks.push(generate_family_impl(&block_families));
    code_blocks.push("}".to_string());
    code_blocks.join("\n")
}