use crate::{
    bloc::Bloc,
    blocs::MAX_HEIGHT,
    chunk,
    chunk::Chunk,
    noise_build::{NoiseFct},
    packed_ints::PackedUsizes,
    terrain_gen::TerrainGen,
    weighted_dist::WeightedPoints, utils::{noise_build::Noise, noise_source::NoiseSource},
};
use array_macro::array;
use itertools::iproduct;
use std::{collections::HashMap, ops::IndexMut, usize, sync::Arc};
const SCALE: f32 = 0.005;
const CHUNK_S1: i32 = chunk::CHUNK_S1 as i32;

pub struct Earth {
    soils: WeightedPoints<Bloc>,
    seed: u32,
    source: Arc<NoiseSource>,
    landratio: f32,
    config: HashMap<String, f32>,
}

impl Earth {
    pub fn sample_col(&self, x: i32, z: i32) -> [Vec<f32>; 3] {
        self.sample( x * CHUNK_S1,
            z * CHUNK_S1,
            CHUNK_S1 as usize,
            CHUNK_S1 as usize,
            SCALE,
            self.seed,
            self.source.clone()
        )
    }
}

impl NoiseFct<3> for Earth {
    fn build(&self, n: &mut Noise<f64>) -> [f32; 3] {
        let land = n.noise(0.7) + n.noise(3.) * 0.3 + n.noise(9.) * 0.1;
        let ocean = land.clone().turn().pos();
        let land = land.norm().mask(self.landratio);
        let mount_mask = (n.noise(1.) + n.noise(2.)*0.3).norm().mask(0.2);
        let mount = (1.-n.noise(1.).abs()) * land.clone() * mount_mask;
        let y = (land + mount).norm();
        // more attitude => less temperature
        let t = y.clone().pow(2).turn() * n.noise(0.2).pos();
        // closer to the ocean => more humidity
        // higher temp => more humidity
        let h = t.clone().sqrt() * (ocean*0.5 + n.noise(0.8).pos()).norm();
        // Add a slope output, useful for rocks and vegetation
        [y.value, t.value, h.value]
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
        let landratio = config.get("land_ratio").copied().unwrap_or(0.5);
        Earth {
            soils: WeightedPoints::from_csv("assets/data/soils_condition.csv").unwrap(),
            seed,
            landratio,
            config,
            source: Arc::new(NoiseSource::new())
        }
    }

    fn gen(&self, col: (i32, i32)) -> [Option<Chunk>; MAX_HEIGHT / chunk::CHUNK_S1] {
        let mut res = array![_ => None; MAX_HEIGHT / chunk::CHUNK_S1];
        let [ys, ts, hs] = self.sample_col(col.0, col.1);
        for (i, (dx, dz)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
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
