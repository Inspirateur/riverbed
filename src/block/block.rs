use std::ops::Range;

use crate::{Block, BlockFamily};

impl Block {
    pub fn friction(&self) -> f32 {
        match self {
            Block::Air => 0.05,
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

    pub fn is_traversable(&self) -> bool {
        match self {
            Block::Air | Block::SeaBlock => true,
            _ => false,
        }
    }

    pub fn is_targetable(&self) -> bool {
        match self {
            Block::Air | Block::SeaBlock => false,
            _ => true
        }
    }
    
    pub fn is_opaque(&self) -> bool {
        if self.is_foliage() {
            return false;
        }
        match self {
            Block::Glass | Block::SeaBlock | Block::Air | Block::Campfire => false,
            _ => true
        }
    }

    pub fn is_foliage(&self) -> bool {
        self.families().contains(&BlockFamily::Leaves)
    }

    pub fn is_fertile_soil(&self) -> bool {
        match self {
            Block::GrassBlock | Block::Podzol | Block::Snow
                => true,
            _ => false
        }
    }
}

pub type Soils = Vec<([Range<f32>; 2], Block)>;
