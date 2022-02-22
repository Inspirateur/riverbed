use crate::range_index::FuzzyIndex;
use bevy::prelude::*;
use itertools::{iproduct, zip};
use noise::{NoiseFn, Seedable, SuperSimplex};
use std::usize;
const LS_NOISES: usize = 4;
const C_NOISES: usize = 2;

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
    a: f64,
}

impl LandShape {
    pub fn new(seed: u32, landratio: f32) -> Self {
        let mut sources = [SuperSimplex::new(); LS_NOISES];
        for x in 0..LS_NOISES {
            sources[x] = SuperSimplex::new().set_seed(seed + x as u32);
        }
        Self {
            noises: sources,
            remap: PieceWiseRemap::new(0.05, vec![(0.9, |x| x.powi(4))]),
            a: (1. - 2. * landratio).clamp(-1., 0.99) as f64,
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
        // continental noise, `a` controls the land/ocean ratio while keeping the range [-1, 1]
        let mut h =
            ((self.noises[0].get(LandShape::mul2(point, 0.4)) - self.a) / (1. - self.a)).max(-1.);
        // mountain noise
        h += (1.0 - self.noises[1].get(LandShape::mul2(point, 1)).abs()) * 0.4
            + (self.noises[2].get(LandShape::mul2(point, 2)).abs()) * 0.2
            + (1.0 - self.noises[3].get(LandShape::mul2(point, 4)).abs()) * 0.05;
        // rescaling to stay in [-1, 1]
        h = h / 1.65;
        // smoothen the surface and sharpen the mountain
        self.remap.apply(h)
    }
}

struct ClimateShape {
    noises: [SuperSimplex; C_NOISES],
}

impl ClimateShape {
    pub fn new(seed: u32) -> Self {
        let mut sources = [SuperSimplex::new(); C_NOISES];
        for x in 0..C_NOISES {
            sources[x] = SuperSimplex::new().set_seed(seed + x as u32);
        }
        Self { noises: sources }
    }
}

impl NoiseFn<[f64; 2]> for ClimateShape {
    fn get(&self, point: [f64; 2]) -> f64 {
        // base noise
        let c = self.noises[0].get(LandShape::mul2(point, 0.5)) + self.noises[1].get(point) * 0.2;
        // rescale in [0, 1]
        0.5 + 0.5 * c / 1.2
    }
}

pub struct Sampler {
    pub data: Vec<f32>,
    noise: Box<dyn NoiseFn<[f64; 2]> + Send + Sync>,
    pub size: u32,
}

impl Sampler {
    fn new(size: u32, noise: impl NoiseFn<[f64; 2]> + Send + Sync + 'static, zoom: f32) -> Self {
        let mut sampler = Sampler {
            data: vec![0.; (size * size) as usize],
            noise: Box::new(noise),
            size: size,
        };
        sampler.resample(zoom);
        sampler
    }

    fn resample(&mut self, zoom: f32) {
        let sizef = self.size as f32;
        let scale = 1. / zoom;
        for (i, (x, y)) in iproduct!(0..self.size, 0..self.size).enumerate() {
            self.data[i] = self.noise.get([
                (scale * x as f32 / sizef) as f64,
                (scale * y as f32 / sizef) as f64,
            ]) as f32;
        }
    }
}

pub struct Zoom(pub f32);

pub struct Earth {
    s_elevation: Sampler,
    s_temperature: Sampler,
    s_humidity: Sampler,
    pub elevation: Vec<f32>,
    pub temperature: Vec<f32>,
    pub humidity: Vec<f32>,
    pub size: u32,
}

impl Earth {
    fn new(size: u32, zoom: f32, seed: u32, landratio: f32) -> Self {
        let s_elevation = Sampler::new(size, LandShape::new(seed, landratio), zoom);
        let s_temperature =
            Sampler::new(size, ClimateShape::new(seed + LS_NOISES as u32), zoom * 4.);
        let s_humidity = Sampler::new(
            size,
            ClimateShape::new(seed + LS_NOISES as u32 + C_NOISES as u32),
            zoom,
        );

        let mut earth = Earth {
            s_elevation: s_elevation,
            s_temperature: s_temperature,
            s_humidity: s_humidity,
            elevation: vec![0.; (size * size) as usize],
            temperature: vec![0.; (size * size) as usize],
            humidity: vec![0.; (size * size) as usize],
            size: size,
        };
        earth.resample(zoom);
        earth
    }

    fn resample(&mut self, zoom: f32) {
        self.s_elevation.resample(zoom);
        self.s_temperature.resample(zoom);
        self.s_humidity.resample(zoom);
        self.elevation.clone_from(&self.s_elevation.data);
        for (i, (y, t)) in zip(&self.s_elevation.data, &self.s_temperature.data).enumerate() {
            // high altitude -> colder temp
            self.temperature[i] = (1. - y.max(0.)).max(0.1) * t;
        }
        for (i, (t, h)) in zip(&self.temperature, &self.s_humidity.data).enumerate() {
            // hot temp -> less humid, cold temp -> not humid
            self.humidity[i] = h * (1. - (t - 0.7).powi(2) * 2.);
        }
    }
}

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        // (color, [temp, hum])
        let mut soils = FuzzyIndex::<[u8; 3], 2>::new();
        // polar
        soils.insert([250, 240, 230], [0.0..0.3, 0.0..0.2]);
        // steppe
        soils.insert([100, 150, 200], [0.3..0.6, 0.0..0.2]);
        // desert
        soils.insert([120, 180, 200], [0.6..1., 0.0..0.2]);
        // tundra
        soils.insert([150, 200, 100], [0.0..0.3, 0.2..0.5]);
        // grassy plains
        soils.insert([100, 200, 50], [0.3..0.7, 0.2..0.5]);
        // savannah
        soils.insert([50, 200, 150], [0.7..1., 0.2..0.5]);
        // snow forest
        soils.insert([60, 100, 20], [0.0..0.3, 0.5..1.]);
        // forest
        soils.insert([80, 150, 50], [0.3..0.7, 0.5..1.]);
        // tropical forest
        soils.insert([100, 200, 150], [0.7..1., 0.5..1.]);
        let initial_zoom = 0.25;
        app.insert_resource(soils)
            .insert_resource(Zoom(initial_zoom))
            .insert_resource(Earth::new(400, initial_zoom, 1, 0.4));
    }
}
