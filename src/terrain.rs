use bevy::prelude::*;
use itertools::{iproduct, zip};
use noise::{NoiseFn, Seedable, SuperSimplex};
use std::usize;
const LS_NOISES: usize = 4;

struct DomainWarp2d {
    noises: [SuperSimplex; 2],
}

impl DomainWarp2d {
    pub fn new(seed: u32) -> Self {
        Self {
            noises: [
                SuperSimplex::new().set_seed(seed),
                SuperSimplex::new().set_seed(seed + 1),
            ],
        }
    }

    pub fn get(&self, point: [f64; 2]) -> [f64; 2] {
        [
            point[0] + self.noises[0].get(point) * 0.5,
            point[1] + self.noises[1].get(point) * 0.5,
        ]
    }
}

struct PieceWiseRemap {
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

struct LandShape {
    noises: [SuperSimplex; LS_NOISES],
    remap: PieceWiseRemap,
}

impl LandShape {
    pub fn new(seed: u32) -> Self {
        let mut sources = [SuperSimplex::new(); LS_NOISES];
        for x in 0..LS_NOISES {
            sources[x] = SuperSimplex::new().set_seed(seed + x as u32);
        }
        Self {
            noises: sources,
            remap: PieceWiseRemap::new(0.05, vec![(0.9, |x| x.powi(3))]),
        }
    }

    fn mul2<T>(point: [f64; 2], s: T) -> [f64; 2]
    where
        T: Into<f64> + Copy,
    {
        [point[0] * s.into(), point[1] * s.into()]
    }
}

impl NoiseFn<[f64; 2]> for LandShape {
    fn get(&self, point: [f64; 2]) -> f64 {
        // rough land shape
        let h = self.noises[0].get(LandShape::mul2(point, 0.4))
            + self.noises[1].get(LandShape::mul2(point, 1.5)) * 0.45
            + (self.noises[2].get(LandShape::mul2(point, 3)).abs()) * 0.3
            + (1.0 - self.noises[3].get(LandShape::mul2(point, 6)).abs()) * 0.15;
        self.remap.apply(h / 1.9 - 0.15)
    }
}

pub struct Heightmap {
    pub data: Vec<f32>,
    noise: Box<dyn NoiseFn<[f64; 2]> + Send + Sync>,
    pub size: u32,
}

impl Heightmap {
    fn new(size: u32, noise: impl NoiseFn<[f64; 2]> + Send + Sync + 'static, zoom: f32) -> Self {
        let mut heightmap = Heightmap {
            data: vec![0.; (size * size) as usize],
            noise: Box::new(noise),
            size: size,
        };
        heightmap.resample(zoom);
        heightmap
    }

    fn resample(&mut self, zoom: f32) {
        let sizef = self.size as f32;
        let scale = 1. / zoom;
        for (i, (x, y)) in iproduct!(0..self.size, 0..self.size).enumerate() {
            self.data[i] = self.noise.get([
                (scale * 2. * (x as f32) / sizef - 1.) as f64,
                (scale * 2. * (y as f32) / sizef - 1.) as f64,
            ]) as f32;
        }
    }
}

pub struct Zoom(pub f32);

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        let initial_zoom = 0.25;
        app.insert_resource(Zoom(initial_zoom))
            .insert_resource(Heightmap::new(400, LandShape::new(3), initial_zoom));
    }
}
