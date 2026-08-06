use rb_block::Block;
use rb_world::CHUNK_S1;

use crate::noise_samples::NoiseSamples;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerTag {
    Mantle,
    Soil,
    Deposit,
    Fixed { height: usize },
}

pub enum Height {
    Constant(f32),
    Noise(usize),
}

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
        }
    }
}
