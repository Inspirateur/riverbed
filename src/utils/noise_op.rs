use std::ops::{Add, BitOr, Div, Mul, Sub};

#[derive(Clone)]
pub enum NoiseOp {
    Noise(f32),
    Const(f32),
    Add(Box<NoiseOp>, Box<NoiseOp>),
    Sub(Box<NoiseOp>, Box<NoiseOp>),
    Mul(Box<NoiseOp>, Box<NoiseOp>),
    Div(Box<NoiseOp>, Box<NoiseOp>),
    Abs(Box<NoiseOp>),
    Pow(Box<NoiseOp>, i32),
    Exp(Box<NoiseOp>),
    Rescale(Box<NoiseOp>, f32, f32),
    Pipe(Box<NoiseOp>, Box<NoiseOp>)
}

impl NoiseOp {
    pub fn abs(self) -> Self {
        NoiseOp::Abs(Box::new(self))
    }

    pub fn rescale(self, min: f32, max: f32) -> Self {
        debug_assert!(min < max);
        NoiseOp::Rescale(Box::new(self), min, max)
    }

    pub fn pow(self, p: i32) -> Self {
        NoiseOp::Pow(Box::new(self), p)
    }

    fn exp(self) -> Self {
        NoiseOp::Exp(Box::new(self))
    }

    pub fn mask(self, t: f32) -> Self {
        let t = 2. * (1. - t) - 1.;
        // a steep sigmoid
        1.0 / (1.0 + (-16. * (self - t)).exp())
    }
}

impl From<f32> for NoiseOp {
    fn from(v: f32) -> Self {
        NoiseOp::Const(v)
    }
}

impl<N: Into<NoiseOp>> Add<N> for NoiseOp {
    type Output = NoiseOp;

    fn add(self, rhs: N) -> Self::Output {
        NoiseOp::Add(Box::new(self), Box::new(rhs.into()))
    }
}

impl<N: Into<NoiseOp>> Mul<N> for NoiseOp {
    type Output = NoiseOp;

    fn mul(self, rhs: N) -> Self::Output {
        NoiseOp::Mul(Box::new(self), Box::new(rhs.into()))
    }
}

impl<N: Into<NoiseOp>> Div<N> for NoiseOp {
    type Output = NoiseOp;

    fn div(self, rhs: N) -> Self::Output {
        NoiseOp::Div(Box::new(self), Box::new(rhs.into()))
    }
}

impl<N: Into<NoiseOp>> Sub<N> for NoiseOp {
    type Output = NoiseOp;

    fn sub(self, rhs: N) -> Self::Output {
        NoiseOp::Sub(Box::new(self), Box::new(rhs.into()))
    }
}

impl BitOr for NoiseOp {
    type Output = NoiseOp;

    fn bitor(self, rhs: NoiseOp) -> Self::Output {
        NoiseOp::Pipe(Box::new(self), Box::new(rhs))
    }
}

// BLANKET IMPLs
impl Add<NoiseOp> for f32 {
    type Output = NoiseOp;

    fn add(self, rhs: NoiseOp) -> Self::Output {
        rhs + self
    }
}

impl Mul<NoiseOp> for f32 {
    type Output = NoiseOp;

    fn mul(self, rhs: NoiseOp) -> Self::Output {
        rhs * self
    }
}

impl Div<NoiseOp> for f32 {
    type Output = NoiseOp;

    fn div(self, rhs: NoiseOp) -> Self::Output {
        NoiseOp::from(self)/rhs
    }
}

impl Sub<NoiseOp> for f32 {
    type Output = NoiseOp;

    fn sub(self, rhs: NoiseOp) -> Self::Output {
        NoiseOp::from(self) - rhs
    }
}