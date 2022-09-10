use std::{
    ops::{Add, BitOr, Div, Mul, Sub},
    sync::Arc,
};

#[derive(Clone)]
pub enum NoiseFn {
    Noise,
    f32,
    Add(Arc<NoiseFn>, Arc<NoiseFn>),
    Mul(Arc<NoiseFn>, Arc<NoiseFn>),
    Div(Arc<NoiseFn>, Arc<NoiseFn>),
    Pow(Arc<NoiseFn>, i32),
}

pub fn noise(freq: f32) -> NoiseFn {
    todo!()
}

impl NoiseFn {
    pub fn seed(self, seed: u32) -> Self {
        todo!()
    }

    pub fn abs(self) -> Self {
        todo!()
    }

    pub fn rescale(self, min: f32, max: f32) -> Self {
        debug_assert!(min < max);
        todo!()
    }

    pub fn pow(self, p: u32) -> Self {
        let mut res = self.clone();
        for _ in 0..p {
            res = res * self.clone();
        }
        res
    }

    fn exp(self) -> Self {
        todo!()
    }

    pub fn mask(self, t: f32) -> Self {
        let t = 2. * (1. - t) - 1.;
        // a steep sigmoid
        1.0 / (1.0 + (-16. * (self - t)).exp())
    }

    pub fn sample(&self, x: i32, y: i32, dist: usize, points: usize) -> Vec<Vec<f32>> {
        todo!()
    }
}

impl Add for NoiseFn {
    type Output = NoiseFn;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Add<f32> for NoiseFn {
    type Output = NoiseFn;

    fn add(self, rhs: f32) -> Self::Output {
        todo!()
    }
}

impl BitOr for NoiseFn {
    type Output = NoiseFn;

    fn bitor(self, rhs: NoiseFn) -> Self::Output {
        todo!()
    }
}

impl Mul for NoiseFn {
    type Output = NoiseFn;

    fn mul(self, rhs: NoiseFn) -> Self::Output {
        todo!()
    }
}

impl Mul<f32> for NoiseFn {
    type Output = NoiseFn;

    fn mul(self, rhs: f32) -> Self::Output {
        todo!()
    }
}

impl Div<NoiseFn> for f32 {
    type Output = NoiseFn;

    fn div(self, rhs: NoiseFn) -> Self::Output {
        todo!()
    }
}

// BLANKET IMPLS
impl Add<NoiseFn> for f32 {
    type Output = NoiseFn;

    fn add(self, rhs: NoiseFn) -> Self::Output {
        rhs + self
    }
}

impl Mul<NoiseFn> for f32 {
    type Output = NoiseFn;

    fn mul(self, rhs: NoiseFn) -> Self::Output {
        rhs * self
    }
}

impl Sub<f32> for NoiseFn {
    type Output = NoiseFn;

    fn sub(self, rhs: f32) -> Self::Output {
        self + -rhs
    }
}

impl Sub<NoiseFn> for f32 {
    type Output = NoiseFn;

    fn sub(self, rhs: NoiseFn) -> Self::Output {
        -self + rhs
    }
}

impl Div<f32> for NoiseFn {
    type Output = NoiseFn;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1.0 / rhs)
    }
}
