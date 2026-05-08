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
            Block::Air | Block::SeaBlock | Block::Ice | Block::Glass | Block::Campfire => false,
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

    /// Mass per 1m³ block. Used by `spawn_voxel_grid` to set the rigid body's
    /// total mass and centre of mass. `0` for traversable blocks (they don't
    /// contribute to a grid). Values are tuned for game feel — relative
    /// ratios approximate real materials (wood < soil < stone < ore) but
    /// magnitudes are kept modest so Avian's solver stays stable.
    pub fn density(&self) -> f32 {
        if matches!(self, Block::Air | Block::SeaBlock) {
            return 0.0;
        }
        if self.is_foliage() {
            return 0.1;
        }
        let families = self.families();
        if families.contains(&BlockFamily::Ore) {
            return 3.0;
        }
        if families.contains(&BlockFamily::Stone) {
            return 2.0;
        }
        if families.contains(&BlockFamily::Crystal) {
            return 0.8;
        }
        if families.contains(&BlockFamily::Wood)
            || families.contains(&BlockFamily::Log)
            || families.contains(&BlockFamily::Planks)
        {
            return 0.6;
        }
        if families.contains(&BlockFamily::Soil) {
            return 1.0;
        }
        // Fall-through for one-off blocks (Campfire, Smelter, Kiln, …).
        1.0
    }
}
