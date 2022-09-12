use crate::noise_op::NoiseOp;
pub struct NoiseFn;

impl NoiseFn {
    pub fn sample(&self, x: i32, y: i32, dist: usize, points: usize, base_scale: f32) -> Vec<Vec<f32>> {
        todo!()
    }
}

pub trait NoiseBuild {
    fn build(self, seed: u32) -> NoiseFn;
}

impl NoiseBuild for NoiseOp {
    fn build(self, seed: u32) -> NoiseFn {
        // TODO: automatic rescaling 
        // a*Noise + b*Noise + c*Noise => (a*Noise + b*Noise + c*Noise)/(a+b+c)
        // TODO: automatic seeding (each Noise must have a different seed)
        // TODO: SIMD optimisations; a+b*c translates into a single fmadd instruction
        todo!()
    }
}