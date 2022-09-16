use crate::{
    blocs::{Bloc, MAX_HEIGHT, CHUNK_S1, Col},
    noise_build::NoiseFct,
    terrain_gen::TerrainGen,
    weighted_dist::WeightedPoints, utils::{noise_build::Noise, noise_source::NoiseSource},
};
use itertools::iproduct;
use std::{collections::HashMap, usize, sync::Arc};
const SCALE: f32 = 0.005;

pub struct Earth {
    soils: WeightedPoints<Bloc>,
    seed: u32,
    source: Arc<NoiseSource>,
    landratio: f32,
    config: HashMap<String, f32>,
}

impl Earth {
    pub fn sample_col(&self, x: i32, z: i32) -> [Vec<f32>; 3] {
        self.sample( x * CHUNK_S1 as i32,
            z * CHUNK_S1 as i32,
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
        let y = (land + mount.clone()).norm();
        // more attitude => less temperature
        let t = mount.pow(2).turn() * n.noise(0.2).pos();
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

    fn gen(&self, col_pos: (i32, i32)) -> Col {
        let mut col = Col::new();
        let [ys, ts, hs] = self.sample_col(col_pos.0, col_pos.1);
        for (i, (dx, dz)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
            let (y, t, h) = (ys[i], ts[i], hs[i]);
            let y = (y * MAX_HEIGHT as f32 * 0.8) as i32;
            assert!(y >= 0);
            col.set((dx, y, dz), self.soils.closest(&[t as f32, h as f32]).0);
            for y_ in (y-3)..y {
                if y_ < 0 {
                    break;
                }
                col.set((dx, y_, dz), Bloc::Dirt);
            }
        }
        // this is a bit too slow so we don't bother with it for now
        // col.fill_up(Bloc::Stone);
        col
    }
}
