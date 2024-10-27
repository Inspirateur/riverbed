use std::{collections::{BTreeMap, BTreeSet}, fmt::Display};
use itertools::Itertools;
use crate::parse::{BlockFlag, BlockFrag, IR};

const BLOCK_FAM: &'static str = "BlockFamily";
const BLOCKS: &'static str = "Block";

fn tab(i: u32) -> String {
    (0..i).map(|_| "\t").collect()
}

struct BlockEntry {
    name: String,
    families: BTreeSet<String>,
    flags: BTreeSet<BlockFlag>,
}

impl PartialEq for BlockEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for BlockEntry {}

impl PartialOrd for BlockEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for BlockEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Display for BlockEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
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

fn generate_enum<T: Display>(name: &str, variants: &BTreeSet<T>) -> String {
    format!(
        "#[derive(Debug, Display, PartialEq, EnumIter, EnumString, Eq, Serialize, Deserialize, Clone, Copy, Hash)]\npub enum {name} {{\n\t{}\n}}\n", 
        variants.into_iter().join(",\n\t")
    )
}

fn generate_family_impl(blocks: &BTreeSet<BlockEntry>) -> String {
    MatchFn::new("families", &format!("Vec<{BLOCK_FAM}>")).with_arms(
        blocks.into_iter().map(
            |block| 
                format!("{BLOCKS}::{} => vec![{}]", block, block.families.iter().map(|f| format!("{BLOCK_FAM}::{f}")).join(", "))
            ).collect::<Vec<_>>()
    ).to_rust(1)
}

fn generate_flags(blocks: &mut BTreeSet<BlockEntry>) -> String {
    let mut flag_fns = BTreeMap::new();
    let mut generated_blocks = BTreeSet::new();
    for block in blocks.iter() {
        for flag in block.flags.clone().into_iter() {
            match flag {
                BlockFlag::Renewable(minutes) => {
                    let depleted_block = BlockEntry {
                        name: format!("Depleted{block}"),
                        families: block.families.clone(),
                        flags: block.flags.clone().into_iter().filter(|f| !matches!(f, BlockFlag::Renewable(_))).collect()
                    };
                    flag_fns.entry("depleted".to_string()).or_insert(MatchFn::new("depleted", &BLOCKS).with_default("*self")).arms.push(
                        format!("{BLOCKS}::{block} => {BLOCKS}::{depleted_block}")
                    );
                    flag_fns.entry("renewed".to_string()).or_insert(MatchFn::new("renewed", &BLOCKS).with_default("*self")).arms.push(
                        format!("{BLOCKS}::{depleted_block} => {BLOCKS}::{block}")
                    );
                    flag_fns.entry("renewal_minutes".to_string()).or_insert(MatchFn::new("renewal_minutes", "Option<u32>").with_default("None")).arms.push(
                        format!("{BLOCKS}::{depleted_block} => Some({minutes})")
                    );
                    generated_blocks.insert(depleted_block);
                },
                BlockFlag::Furnace(temperature) => {
                    let lit_furnace = BlockEntry {
                        name: format!("{block}On"),
                        families: block.families.clone(),
                        flags: block.flags.clone()
                    };
                    flag_fns.entry("on".to_string()).or_insert(MatchFn::new("on", &BLOCKS).with_default("*self")).arms.push(
                        format!("{BLOCKS}::{block} => {BLOCKS}::{lit_furnace}")
                    );
                    flag_fns.entry("off".to_string()).or_insert(MatchFn::new("off", &BLOCKS).with_default("*self")).arms.push(
                        format!("{BLOCKS}::{lit_furnace} => {BLOCKS}::{block}")
                    );
                    flag_fns.entry("furnace_temp".to_string()).or_insert(MatchFn::new("furnace_temp", "Option<u32>").with_default("None")).arms.push(
                        format!("{BLOCKS}::{block} | {BLOCKS}::{lit_furnace} => Some({temperature})")
                    );
                    generated_blocks.insert(lit_furnace);
                },
                _ => {
                    let flag_name = format!("is_{:?}", flag).to_lowercase();
                    flag_fns.entry(flag_name.clone()).or_insert(MatchFn::new(&flag_name, "u32").with_default("true")).arms.push(
                        format!("{BLOCKS}::{block} => true")
                    );
                }
            }
        }
    }
    blocks.extend(generated_blocks);
    flag_fns.values().map(|match_fn| match_fn.to_rust(1)).join("\n\n")
}

pub fn generate(ir: &IR) -> String {
    let mut blocks: BTreeSet<BlockEntry> = BTreeSet::new();
    for block_pattern in ir.decl.iter() {
        let families = block_pattern.0.0.iter().filter_map(|frag| match frag { 
            BlockFrag::Ident(_) => None,
            BlockFrag::SetName(set_name) => Some(set_name.clone()) 
        }).collect::<BTreeSet<_>>();
        for frags in block_pattern.0.0.iter()
            .map(|frag| match frag {
                BlockFrag::Ident(ident) => vec![ident],
                BlockFrag::SetName(set_name) => ir.sets.get(set_name).unwrap().iter().collect()
            }).multi_cartesian_product()
        {
            let block: String = frags.into_iter().map(|s| s.as_str()).collect();
            blocks.insert(BlockEntry {
                name: block,
                families: families.clone(),
                flags: block_pattern.0.1.clone()
            });
        }
    }
    let flag_code = generate_flags(&mut blocks);
    let mut code_blocks = Vec::new();
    code_blocks.push("use serde::{Deserialize, Serialize};".to_string());
    code_blocks.push("use strum_macros::{EnumIter, EnumString, Display};".to_string());
    code_blocks.push(String::new());
    code_blocks.push(generate_enum(BLOCK_FAM, &ir.sets.keys().map(|s| s.to_owned()).collect()));
    for (family, variants) in ir.sets.iter() {
        code_blocks.push(generate_enum(family, variants));
    }
    code_blocks.push(generate_enum(BLOCKS, &blocks));
    code_blocks.push(format!("impl {BLOCKS} {{"));
    code_blocks.push(flag_code);
    code_blocks.push(generate_family_impl(&blocks));
    code_blocks.push("}".to_string());
    code_blocks.join("\n")
}