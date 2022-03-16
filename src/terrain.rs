use crate::{piecewise_remap::PieceWiseRemap, range_index::FuzzyIndex, sampler::Sampler};
use bevy::prelude::*;
use itertools::zip;
use noise::{NoiseFn, Seedable, SuperSimplex};
use std::usize;
const LS_NOISES: usize = 4;
const C_NOISES: usize = 2;

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
            temperature: vec![0.; (size * size) as usize],
            humidity: vec![0.; (size * size) as usize],
            size: size,
        };
        earth.resample(zoom);
        earth
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
    }
}

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        // (color, [temp, hum])
        let mut soils = FuzzyIndex::<[u8; 3], 2>::new();
        // polar
        soils.insert([250, 230, 210], [0.0..0.2, 0.0..0.2]);
        // steppe
        soils.insert([100, 150, 150], [0.2..0.6, 0.0..0.1]);
        // desert
        soils.insert([120, 200, 220], [0.6..1., 0.0..0.2]);
        // tundra
        soils.insert([200, 200, 250], [0.0..0.2, 0.2..0.4]);
        // grassy plains
        soils.insert([150, 200, 100], [0.2..0.7, 0.1..0.3]);
        // dry plains
        soils.insert([50, 150, 120], [0.7..1., 0.2..0.3]);
        // snow forest
        soils.insert([240, 240, 240], [0.0..0.3, 0.3..1.]);
        // forest
        soils.insert([80, 150, 50], [0.3..0.7, 0.3..8.]);
        // savannah
        soils.insert([100, 200, 220], [0.7..1., 0.3..7.]);
        // marsh
        soils.insert([80, 150, 150], [0.4..0.7, 0.8..1.]);
        // tropical forest
        soils.insert([100, 200, 150], [0.7..1., 0.7..1.]);
        let initial_zoom = 0.1;
        app.insert_resource(soils)
            .insert_resource(Zoom(initial_zoom))
            .insert_resource(Earth::new(400, initial_zoom, 1, 0.35));
    }
}
