use crate::{world::CHUNK_S1, Block};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerTag {
    Mantle,
    Soil, 
    Deposit,
    Fixed {
        height: usize
    }
}

pub enum Height {
    Constant(f32),
    Noise(Vec<f32>),
}

pub struct Layer {
    pub block: Block,
    pub height: Height,
    pub tag: LayerTag
}

impl Layer {
    pub fn height(&self, dx: usize, dz: usize) -> f32 {
        match &self.height {
            Height::Constant(h) => *h,
            Height::Noise(noise) => noise[dx + dz * CHUNK_S1],
        }
    }
}