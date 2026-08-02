use rb_block::Block;
use rb_world::CHUNK_S1;
use strum_macros::EnumIter;

use crate::noise_samples::NoiseSamples;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum LayerTag {
    Mantle,
    Soil,
    Deposit,
    Liquid,
    Floating,
}

impl LayerTag {
    pub fn should_interpolate(&self) -> bool {
        match self {
            LayerTag::Mantle => true,
            LayerTag::Soil => true,
            LayerTag::Deposit => true,
            LayerTag::Liquid => false,
            LayerTag::Floating => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Height {
    Constant(f32),
    Noise(usize),
    OffsetNoise(usize, f32),
}

#[derive(Debug, Clone, Copy)]
pub struct Layer {
    pub block: Block,
    pub height: Height,
    pub tag: LayerTag,
}

impl Layer {
    pub fn height(&self, noise_sample: &NoiseSamples, dx: usize, dz: usize) -> f32 {
        match &self.height {
            Height::Constant(h) => *h,
            Height::Noise(i) => noise_sample[*i][dx + dz * CHUNK_S1],
            Height::OffsetNoise(i, offset) => noise_sample[*i][dx + dz * CHUNK_S1] + offset,
        }
    }
}
