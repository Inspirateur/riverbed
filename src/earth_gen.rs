use crate::{
    bloc::Bloc,
    blocs::MAX_HEIGHT,
    chunk,
    chunk::Chunk,
    noise_op::{NoiseOp::Noise},
    noise_build::{NoiseFn, NoiseBuild},
    packed_ints::PackedUsizes,
    terrain_gen::TerrainGen,
    weighted_dist::WeightedPoints,
};
use array_macro::array;
use itertools::iproduct;
use std::{collections::HashMap, ops::IndexMut, usize};
const SCALE: f32 = 0.01;
const CHUNK_S1: i32 = chunk::CHUNK_S1 as i32;

pub struct Earth {
    noise: NoiseFn,
    soils: WeightedPoints<Bloc>,
    seed: u32,
    config: HashMap<String, f32>,
}

impl Earth {
    pub fn sample_col(&self, x: i32, z: i32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let mut noise = self.noise.sample(
            x * CHUNK_S1,
            z * CHUNK_S1,
            CHUNK_S1 as usize,
            CHUNK_S1 as usize,
            SCALE
        );
        (noise.remove(0), noise.remove(0), noise.remove(0))
    }

    pub fn build(seed: u32, landratio: f32) -> NoiseFn {
        let land = Noise(1.) + Noise(0.3) * 0.3 + Noise(0.1) * 0.1;
        let land = land.mask(landratio);
        let mount_mask = Noise(1.).mask(0.2);
        let mount = (Noise(4.).abs() + Noise(8.).abs() * 0.3) * land.clone() * mount_mask;
        let y = land * 0.2 + mount;
        let t = (1. - y.clone()).rescale(0.5, 1.0) * Noise(1.);
        let h = (1. - (t.clone() - 0.7).pow(2) * 2.) * Noise(1.);
        (y | t | h).build(seed)
    }
}

impl Clone for Earth {
    fn clone(&self) -> Self {
        Earth::new(self.seed, self.config.clone())
    }
}

impl TerrainGen for Earth {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized,
    {
        let landratio = config.get("land_ratio").copied().unwrap_or(0.35);
        Earth {
            noise: Earth::build(seed, landratio),
            soils: WeightedPoints::from_csv("assets/data/soils_condition.csv").unwrap(),
            seed,
            config,
        }
    }

    fn gen(&self, col: (i32, i32)) -> [Option<Chunk>; MAX_HEIGHT / chunk::CHUNK_S1] {
        let mut res = array![_ => None; MAX_HEIGHT / chunk::CHUNK_S1];
        let (ys, ts, hs) = self.sample_col(col.0, col.1);
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let i = (dx + dz * CHUNK_S1) as usize;
            let (y, t, h) = (ys[i], ts[i], hs[i]);
            let y = (y * MAX_HEIGHT as f32 * 0.8) as i32;
            assert!(y >= 0);
            let (qy, dy) = (y / CHUNK_S1, y % CHUNK_S1);
            if res[qy as usize].is_none() {
                res[qy as usize] = Some(Chunk::<PackedUsizes>::new());
            }
            if let Some(chunk) = res.index_mut(qy as usize) {
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
        let mut qy = 0;
        while res[qy].is_none() {
            res[qy] = Some(Chunk::<PackedUsizes>::filled(Bloc::Stone));
            qy += 1;
        }
        res
    }
}
