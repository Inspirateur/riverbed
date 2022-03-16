use itertools::zip;

pub struct PieceWiseRemap {
    min_h: f64,
    h_fns: Vec<(f64, fn(f64) -> f64)>,
    coefs: Vec<(f64, f64)>,
}

impl PieceWiseRemap {
    pub fn new(min_h: f64, h_fns: Vec<(f64, fn(f64) -> f64)>) -> Self {
        let mut coefs = Vec::new();
        let mut a = min_h;
        for (b, h_fn) in &h_fns {
            let fa = h_fn(a);
            let fb = h_fn(*b);
            coefs.push(((fb * a - b * fa) / (fb - fa), (b - a) / (fb - fa)));
            a = *b;
        }
        Self {
            min_h: min_h,
            h_fns: h_fns,
            coefs: coefs,
        }
    }

    pub fn apply(&self, x: f64) -> f64 {
        if x <= self.min_h {
            return x;
        }
        for ((h, h_fn), (a, b)) in zip(&self.h_fns, &self.coefs) {
            if x < *h {
                return a + b * h_fn(x);
            }
        }
        x
    }
}
