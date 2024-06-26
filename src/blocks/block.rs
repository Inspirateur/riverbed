use std::ops::Range;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BlockFamily {
    Stone,
    Log,
    Foliage,
    Soil,
}

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
#[derive(EnumString)]
pub enum Block {
    #[default]
    Air,
    AcaciaLeaves,
    AcaciaLog,
    Bedrock,
    BirchLeaves,
    BirchLog,
    CoarseDirt,
    Cobblestone,
    Dirt,
    Endstone,
    Glass,
    Granite,
    GrassBlock,
    Ice,
    Limestone,
    Mud,
    OakLeaves,
    OakLog,
    Podzol,
    Sand,
    SequoiaLeaves,
    SequoiaLog,
    SpruceLeaves,
    SpruceLog,
    Snow,
    SeaBlock,
}

impl Block {
    pub fn friction(&self) -> f32 {
        match self {
            Block::Air => 0.1,
            Block::Ice => 0.05,
            _ => 1.
        }
    }

    pub fn slowing(&self) -> f32 {
        match self {
            Block::Mud => 0.8,
            _ => 1.
        }
    }

    pub fn traversable(&self) -> bool {
        match self {
            Block::Air | Block::SeaBlock => true,
            _ => false,
        }
    }

    pub fn targetable(&self) -> bool {
        match self {
            Block::Air | Block::SeaBlock => false,
            _ => true
        }
    }
    
    pub fn is_transluscent(&self) -> bool {
        if self.is_foliage() {
            return true;
        }
        match self {
            Block::Glass | Block::SeaBlock => true,
            _ => false
        }
    }

    pub fn is_foliage(&self) -> bool {
        match self {
            Block::OakLeaves | Block::BirchLeaves | Block::SpruceLeaves | Block::SequoiaLeaves
                => true,
            _ => false
        }
    }

    pub fn is_fertile_soil(&self) -> bool {
        match self {
            Block::GrassBlock | Block::Podzol | Block::Snow
                => true,
            _ => false
        }
    }

    pub fn families(&self) -> Vec<BlockFamily> {
        if self.is_foliage() {
            return vec![BlockFamily::Foliage]
        }
        match self {
            Block::Granite | Block::Cobblestone | Block::Endstone | Block::Limestone
                => vec![BlockFamily::Stone],
            Block::OakLog | Block::AcaciaLog | Block::BirchLog | Block::SpruceLog | Block::SequoiaLog
                => vec![BlockFamily::Log],
            Block::Dirt | Block::CoarseDirt | Block::GrassBlock | Block::Sand
                => vec![BlockFamily::Soil],
            _ => vec![]
        }
    }
}

pub type Soils = Vec<([Range<f32>; 2], Block)>;
