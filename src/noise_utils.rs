use itertools::iproduct;
use itertools::zip;
use noise::NoiseFn;

pub fn get_warped_fn<F>(
    point: [f64; 2],
    noise: impl NoiseFn<[f64; 3]>,
    warp: F,
    strength: f64,
) -> f64
where
    F: Fn([f64; 2]) -> f64,
{
    noise.get([point[0], point[1], warp(mul2(point, strength))])
}

pub fn get_warped(
    point: [f64; 2],
    noise: impl NoiseFn<[f64; 3]>,
    warp: impl NoiseFn<[f64; 2]>,
    strength: f64,
) -> f64 {
    get_warped_fn(point, noise, |p| warp.get(p), strength)
}

pub fn mul2<T>(point: [f64; 2], s: T) -> [f64; 2]
where
    T: Into<f64> + Copy,
{
    [point[0] * s.into(), point[1] * s.into()]
}

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

pub struct Sampler {
    pub data: Vec<f32>,
    noise: Box<dyn NoiseFn<[f64; 2]> + Send + Sync>,
    pub size: u32,
    pub zoom: f32,
}

impl Sampler {
    pub fn new(
        size: u32,
        noise: impl NoiseFn<[f64; 2]> + Send + Sync + 'static,
        zoom: f32,
    ) -> Self {
        let mut sampler = Sampler {
            data: vec![0.; (size * size) as usize],
            noise: Box::new(noise),
            size,
            zoom,
        };
        sampler.do_resample(zoom);
        sampler
    }

    fn do_resample(&mut self, zoom: f32) {
        let sizef = zoom * self.size as f32;
        for (i, (x, y)) in iproduct!(0..self.size, 0..self.size).enumerate() {
            self.data[i] = self
                .noise
                .get([(x as f32 / sizef) as f64, (y as f32 / sizef) as f64])
                as f32;
        }
        self.zoom = zoom;
    }

    pub fn resample(&mut self, zoom: f32) {
        if self.zoom != zoom {
            self.do_resample(zoom);
        }
    }
}
