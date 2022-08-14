use crate::{
    noise_utils::{get_warped_fn, mul2, PieceWiseRemap, Sampler},
    weighted_dist::WeightedPoints,
};
use bevy::prelude::*;
use itertools::zip;
use noise::{NoiseFn, Seedable, SuperSimplex};
use std::usize;
const LS_NOISES: usize = 4;
const C_NOISES: usize = 2;
const CONT_S: f64 = 0.2;

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
            remap: PieceWiseRemap::new(0., vec![(0.9, |x| x.powi(2))]),
            a: (1. - 2. * landratio).clamp(-1., 0.99) as f64,
        }
    }
}

impl NoiseFn<[f64; 2]> for LandShape {
    fn get(&self, point: [f64; 2]) -> f64 {
        // continental noise
        let mut h = get_warped_fn(
            mul2(point, CONT_S),
            self.noises[0],
            |p| self.noises[0].get(p).powi(2),
            1.,
        );
        h += (1. - self.noises[1].get(mul2(point, 0.8))) * h * 0.2;
        h /= 1.2;
        h += (1. - self.noises[2].get(mul2(point, 2.)).powi(2)) * h * 0.2;
        h /= 1.2;
        // `a` controls the land/ocean ratio while keeping the range [-1, 1]
        h = ((h - self.a) / (1. - self.a)).max(-1.);
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
        let c = self.noises[0].get(mul2(point, 0.5)) + self.noises[1].get(point) * 0.2;
        // rescale in [0, 1]
        0.5 + 0.5 * c / 1.2
    }
}

pub struct Zoom(pub f32);

pub struct Earth {
    s_elevation: Sampler,
    s_temperature: Sampler,
    s_humidity: Sampler,
    pub elevation: Vec<f32>,
    pub slope: Vec<f32>,
    pub temperature: Vec<f32>,
    pub humidity: Vec<f32>,
    pub size: u32,
}

impl Earth {
    pub fn new(size: u32, zoom: f32, seed: u32, landratio: f32) -> Self {
        let s_elevation = Sampler::new(size, LandShape::new(seed, landratio), zoom);
        let s_temperature =
            Sampler::new(size, ClimateShape::new(seed + LS_NOISES as u32), zoom * 3.);
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
            slope: vec![0.; (size * size) as usize],
            temperature: vec![0.; (size * size) as usize],
            humidity: vec![0.; (size * size) as usize],
            size: size,
        };
        earth.resample(zoom);
        earth
    }

    fn slope(&self, i: usize) -> f32 {
        let size = self.s_elevation.size as usize;
        let dxl = &self.elevation[i] - self.elevation[if i % size > 0 { i - 1 } else { i }];
        let dxr = self.elevation[if i % size < size - 1 { i + 1 } else { i }] - &self.elevation[i];
        let dyl = &self.elevation[i] - self.elevation[if i / size > 0 { i - size } else { i }];
        let dyr =
            self.elevation[if i / size < size - 1 { i + size } else { i }] - &self.elevation[i];
        (dxl + dxr + dyl + dyr) / (4. * self.s_elevation.zoom)
    }

    pub fn resample(&mut self, zoom: f32) {
        let zoom_r = zoom / self.s_elevation.zoom;
        self.s_elevation.resample(zoom);
        self.s_temperature
            .resample(self.s_temperature.zoom * zoom_r);
        self.s_humidity.resample(self.s_humidity.zoom * zoom_r);
        self.elevation.clone_from(&self.s_elevation.data);
        for (i, (y, t)) in zip(&self.s_elevation.data, &self.s_temperature.data).enumerate() {
            // high altitude -> colder temp
            self.temperature[i] = (1. - y.max(0.)).max(0.1) * t;
        }
        for (i, (t, h)) in zip(&self.temperature, &self.s_humidity.data).enumerate() {
            // hot temp -> less humid, cold temp -> not humid
            self.humidity[i] = h * (1. - (t - 0.7).powi(2) * 2.);
        }
        for (i, y) in self.elevation.iter().enumerate() {
            self.slope[i] = if *y <= 0. { 0. } else { self.slope(i) };
        }
    }
}

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        let soils = WeightedPoints::<String>::from_csv("assets/data/soils_conditions.csv");

        let initial_zoom = 0.1;
        app.insert_resource(soils)
            .insert_resource(Zoom(initial_zoom))
            .insert_resource(Earth::new(400, initial_zoom, 1, 0.35));
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
