use std::collections::{BTreeMap, BTreeSet};
use itertools::Itertools;
use crate::parse::{BlockFlag, BlockFrag, IR};

const BLOCK_FAM: &'static str = "BlockFamily";
const BLOCKS: &'static str = "Block";

fn tab(i: u32) -> String {
    (0..i).map(|_| "\t").collect()
}

struct MatchFn {
    name: String,
    return_type: String,
    pub arms: Vec<String>,
    default: Option<String>,
}

impl MatchFn {
    pub fn new(name: &str, return_type: &str) -> Self {
        Self {
            name: name.to_string(), return_type: return_type.to_string(),
            arms: Vec::new(), default: None
        }
    }

    pub fn with_arms(self, arms: Vec<String>) -> Self {
        Self {
            name: self.name, return_type: self.return_type, arms, default: self.default
        }
    }

    pub fn with_default(self, default: &str) -> Self {
        Self {
            name: self.name, return_type: self.return_type, arms: self.arms, default: Some(default.to_string())
        }
    }

    pub fn to_rust(&self, indentation: u32) -> String {
        let i = indentation;
        let mut arms = self.arms.clone();
        if let Some(default) = &self.default {
            arms.push(format!("_ => {}", default));
        }
        format!(
            "{}pub fn {}(&self) -> {} {{\n{}match self {{\n{}{}\n{}}}\n\t}}",
            tab(i), self.name, self.return_type, tab(i+1), tab(i+2), arms.join(&format!(",\n{}", tab(i+2))), tab(i+1)
        )
    }
}

pub fn generate_enum(name: &str, variants: &BTreeSet<String>) -> String {
    format!(
        "#[derive(Debug, PartialEq, EnumString, Eq, Serialize, Deserialize, Clone, Copy, Hash)]\npub enum {name} {{\n\t{}\n}}\n", 
        variants.into_iter().join(",\n\t")
    )
}

pub fn generate_family_impl(block_families: &BTreeMap<String, BTreeSet<&String>>) -> String {
    MatchFn::new("families", &format!("Vec<{BLOCK_FAM}>")).with_arms(
        block_families.into_iter().map(
            |(block, families)| 
                format!("{BLOCKS}::{} => vec![{}]", block, families.into_iter().map(|f| format!("{BLOCK_FAM}::{f}")).join(", "))
            ).collect::<Vec<_>>()
    ).to_rust(1)
}

pub fn generate_flags(blocks: &BTreeSet<(String, BTreeSet<BlockFlag>)>) -> String {
    let mut flag_fns = BTreeMap::new();
    for (block, flags) in blocks.into_iter() {
        for flag in flags.into_iter() {
            match flag {
                BlockFlag::Renewable(minutes) => {
                    flag_fns.entry("depleted".to_string()).or_insert(MatchFn::new("depleted", &BLOCKS).with_default("*self")).arms.push(
                        format!("{BLOCKS}::{block} => {BLOCKS}::Depleted{block}")
                    );
                    flag_fns.entry("renewed".to_string()).or_insert(MatchFn::new("renewed", &BLOCKS).with_default("*self")).arms.push(
                        format!("{BLOCKS}::Depleted{block} => {BLOCKS}::{block}")
                    );
                    flag_fns.entry("renewal_minutes".to_string()).or_insert(MatchFn::new("renewal_minutes", "u32").with_default("0")).arms.push(
                        format!("{BLOCKS}::Depleted{block} => {minutes}")
                    );
                },
                _ => {
                    let flag_name = format!("is_{:?}", flag).to_lowercase();
                    flag_fns.entry(flag_name.clone()).or_insert(MatchFn::new(&flag_name, "u32").with_default("true")).arms.push(
                        format!("{BLOCKS}::Depleted{block} => true")
                    );
                }
            }
        }
    }
    flag_fns.values().map(|match_fn| match_fn.to_rust(1)).join("\n\n")
}

fn remove_all<F, T: Ord>(collection: &mut BTreeSet<T>, mut predicate: F) -> bool
    where F: FnMut(&T) -> bool
{
    let size = collection.len();
    collection.retain(|e| !predicate(e));
    size != collection.len()
}

pub fn generate(ir: &IR) -> String {
    let mut blocks: BTreeSet<String> = BTreeSet::new();
    let mut block_families: BTreeMap<String, BTreeSet<&String>> = BTreeMap::new();
    let mut block_flags: BTreeSet<(String, BTreeSet<BlockFlag>)> = BTreeSet::new();
    for block_pattern in ir.decl.iter() {
        let families = block_pattern.0.0.iter().filter_map(|frag| match frag { 
            BlockFrag::Ident(_) => None,
            BlockFrag::SetName(set_name) => Some(set_name) 
        }).collect::<BTreeSet<_>>();
        for frags in block_pattern.0.0.iter()
            .map(|frag| match frag {
                BlockFrag::Ident(ident) => vec![ident],
                BlockFrag::SetName(set_name) => ir.sets.get(set_name).unwrap().iter().collect()
            }).multi_cartesian_product()
        {
            let block: String = frags.into_iter().map(|s| s.as_str()).collect();
            let mut flags = block_pattern.0.1.clone();
            block_flags.insert((block.clone(), flags.clone()));
            block_families.insert(block.clone(), families.clone());
            blocks.insert(block.clone());
            if remove_all(&mut flags, |f| matches!(f, BlockFlag::Renewable(_))) {
                let depleted_block = format!("Depleted{}", block);
                block_flags.insert((depleted_block.clone(), flags));
                block_families.insert(depleted_block.clone(), families.clone());
                blocks.insert(depleted_block);
            }
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
    code_blocks.push(generate_flags(&block_flags));
    code_blocks.push(generate_family_impl(&block_families));
    code_blocks.push("}".to_string());
    code_blocks.join("\n")
}