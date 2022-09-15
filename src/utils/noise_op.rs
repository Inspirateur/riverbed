use std::ops::{Add, Mul, Div, Sub};


#[derive(Clone)]
pub struct Signal<F> {
    pub value: F,
    ampl: f32,
}

impl Signal<f32> {
    pub fn new(value: f32) -> Self {
        Signal { value, ampl: 1. }
    }

    pub fn abs(self) -> Self {
        Signal {
            value: self.value.abs(),
            ampl: self.ampl
        }
    }

    pub fn pow(self, p: i32) -> Self {
        Signal { value: self.value.powi(p), ampl: self.ampl.powi(p) }
    }

    pub fn sqrt(self) -> Self {
        Signal { value: self.value.sqrt(), ampl: self.ampl.sqrt() }
    }

    fn exp(self) -> Self {
        Signal { value: self.value.exp(), ampl: self.ampl }
    }

    pub fn mask(self, t: f32) -> Self {
        let t = 2. * (1. - t) - 1.;
        // a steep sigmoid
        let mut res = 1.0 / (1.0 + (-16. * (self - t)).exp());
        res.ampl = 1.;
        res
    }

    pub fn norm(self) -> Self {
        Signal { value: self.value/self.ampl, ampl: 1.0 }
    }

    pub fn turn(self) -> Self {
        Signal { value: 1.-self.value, ampl: self.ampl }
    }

    pub fn pos(self) -> Self {
        Signal { value: self.value*0.5+0.5, ampl: self.ampl }
    }
}

// SIGNAL OP
impl<F: Add<Output = F>> Add for Signal<F> {
    type Output = Signal<F>;

    fn add(self, rhs: Signal<F>) -> Self::Output {
        Signal {
            value: self.value+rhs.value,
            ampl: self.ampl+rhs.ampl,
        }
    }
}

impl<F: Mul<Output = F>> Mul for Signal<F> {
    type Output = Signal<F>;

    fn mul(self, rhs: Signal<F>) -> Self::Output {
        Signal {
            value: self.value*rhs.value,
            ampl: self.ampl*rhs.ampl,
        }
    }
}

// SCALAR OP
impl<F: Add<f32, Output = F>> Add<f32> for Signal<F> {
    type Output = Signal<F>;

    fn add(self, rhs: f32) -> Self::Output {
        Signal {
            value: self.value + rhs,
            ampl: self.ampl,
        }
    }
}

impl<F: Mul<f32, Output = F>> Mul<f32> for Signal<F> {
    type Output = Signal<F>;

    fn mul(self, rhs: f32) -> Self::Output {
        Signal {
            value: self.value*rhs,
            ampl: self.ampl*rhs.abs(),
        }
    }
}

impl<F: Sub<f32, Output = F>> Sub<f32> for Signal<F> {
    type Output = Signal<F>;

    fn sub(self, rhs: f32) -> Self::Output {
        Signal {
            value: self.value-rhs,
            ampl: self.ampl
        }
    }
}

impl<F> Sub<Signal<F>> for f32
where f32: Sub<F, Output = F> {
    type Output = Signal<F>;

    fn sub(self, rhs: Signal<F>) -> Self::Output {
        Signal {
            value: self-rhs.value,
            ampl: rhs.ampl
        }
    }
}

impl<F> Div<Signal<F>> for f32 
where f32: Div<F, Output = F> {
    type Output = Signal<F>;

    fn div(self, rhs: Signal<F>) -> Self::Output {
        Signal {
            value: self/rhs.value,
            ampl: rhs.ampl
        }
    }
}

// COMMUTATION
impl<F: Add<f32, Output = F>> Add<Signal<F>> for f32 {
    type Output = Signal<F>;

    fn add(self, rhs: Signal<F>) -> Self::Output {
        rhs + self
    }
}

impl<F: Mul<f32, Output = F>> Mul<Signal<F>> for f32 {
    type Output = Signal<F>;

    fn mul(self, rhs: Signal<F>) -> Self::Output {
        rhs * self
    }
}