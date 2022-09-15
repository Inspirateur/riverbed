use array_macro::array;
use itertools::iproduct;
use crate::noise_op::Signal;
use crate::noise_source::NoiseSource;
use std::sync::Arc;
// TODO impl scalar version for now but change to SIMD instructions when it's more mature

pub struct Noise<F> {
    pub seed: u32,
    pub x: F,
    pub y: F,
    pub source: Arc<NoiseSource>,
}

impl Noise<f64> {
    pub fn noise(&mut self, freq: f32) -> Signal<f32> {
        let res = self.source.get(self.seed, self.x*freq as f64, self.y*freq as f64);
        self.seed += 1;
        Signal::new(res as f32)
    }
}

pub trait NoiseFct<const N: usize> {
    fn build(&self, n: &mut Noise<f64>) -> [f32; N];

    fn sample(
        &self, x: i32, y: i32, dist: usize, points: usize, 
        base_scale: f32, seed: u32, source: Arc<NoiseSource>
    ) -> [Vec<f32>; N] {
        let (x, y, dist) = (x as f64, y as f64, dist as f64);
        let unit = dist/points as f64;
        let mut n = Noise {
            seed, x, y, source
        };
        let mut res = array![_ => vec![0.; points*points]; N];
        for (i, (dx, dy)) in iproduct!(0..points, 0..points).enumerate() {
            n.x = (x + dx as f64*unit)*base_scale as f64;
            n.y = (y + dy as f64*unit)*base_scale as f64;
            n.seed = seed;
            for (j, v) in self.build(&mut n).into_iter().enumerate() {
                res[j][i] = v;
            }
        }
        res
    }
}