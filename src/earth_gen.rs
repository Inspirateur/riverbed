use crate::{
    bloc::Bloc,
    chunk,
    chunk::Chunk,
    noise_utils::{get_warped_fn, mul2, PieceWiseRemap},
    terrain_gen::TerrainGen,
    weighted_dist::WeightedPoints,
};
use bevy::prelude::*;
use itertools::iproduct;
use noise::{NoiseFn, Seedable, SuperSimplex};
use std::{collections::HashMap, usize};
const LS_NOISES: usize = 4;
const C_NOISES: usize = 2;
const CONT_S: f64 = 0.2;
const SCALE: f64 = 0.01;
const CHUNK_S1: i32 = chunk::CHUNK_S1 as i32;
struct LandShape {
    noises: [SuperSimplex; LS_NOISES],
    remap: PieceWiseRemap,
    a: f64,
}

impl LandShape {
    pub fn new(seed: u32, landratio: f64) -> Self {
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

pub struct Zoom(pub f64);

pub struct Earth {
    elevation: Box<dyn NoiseFn<[f64; 2]> + Sync + Send>,
    temperature: Box<dyn NoiseFn<[f64; 2]> + Sync + Send>,
    humidity: Box<dyn NoiseFn<[f64; 2]> + Sync + Send>,
    soils: WeightedPoints<Bloc>,
}

impl Earth {
    pub fn get(&self, x: i32, z: i32, zoom: f64) -> (f64, f64, f64) {
        let point = [x as f64 * SCALE * zoom, z as f64 * SCALE * zoom];
        let y = self.elevation.get(point);
        let t = (1. - y.max(0.)).max(0.1) * self.temperature.get(point);
        let h = (1. - (t - 0.7).powi(2) * 2.) * self.humidity.get(point);
        (y, t, h)
    }
}

impl TerrainGen for Earth {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized,
    {
        let landratio = config.get("land_ratio").copied().unwrap_or(0.35);
        let elevation = Box::new(LandShape::new(seed, landratio as f64));
        let temperature = Box::new(ClimateShape::new(seed + LS_NOISES as u32));
        let humidity = Box::new(ClimateShape::new(seed + LS_NOISES as u32 + C_NOISES as u32));
        Earth {
            elevation,
            temperature,
            humidity,
            soils: WeightedPoints::from_csv("assets/data/soils_condition.csv").unwrap(),
        }
    }

    fn gen(&self, col: (i32, i32)) -> HashMap<i32, Chunk> {
        let mut res = HashMap::new();
        let cx = col.0 * CHUNK_S1;
        let cz = col.1 * CHUNK_S1;
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let (y, t, h) = self.get(cx + dx, cz + dz, 1.);
            let y = (y * 255.) as i32;
            let (cy, dy) = (y / CHUNK_S1, y % CHUNK_S1);
            if !res.contains_key(&cy) {
                res.insert(cy, Chunk::<Vec<usize>>::new());
            }
            if let Some(chunk) = res.get_mut(&cy) {
                chunk.set(
                    dx as usize,
                    dy as usize,
                    dz as usize,
                    self.soils.closest(&[t as f32, h as f32]).0,
                );
                for dy_ in 0..dy as usize {
                    chunk.set(dx as usize, dy_, dz as usize, Bloc::Dirt);
                }
            }
        }
        res.drain()
            .into_iter()
            .map(|(k, v)| (k, Chunk::from(v)))
            .collect()
    }
}
pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        let soils = WeightedPoints::<String>::from_csv("assets/data/soils_condition.csv").unwrap();

        let initial_zoom = 0.1;
        app.insert_resource(soils)
            .insert_resource(Zoom(initial_zoom));
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
