use crate::{
    blocs::{Bloc, Chunk, MAX_HEIGHT},
    packed_ints::PackedUsizes,
    terrain_gen::TerrainGen,
    weighted_dist::WeightedPoints,
};
use array_macro::array;
use itertools::iproduct;
use std::{collections::HashMap, ops::IndexMut};
const CHUNK_S1: i32 = chunk::CHUNK_S1 as i32;

pub struct DebugGen {
    seed: u32,
    config: HashMap<String, f32>,
    soils: WeightedPoints<Bloc>,
}

impl Clone for DebugGen {
    fn clone(&self) -> Self {
        DebugGen::new(self.seed, self.config.clone())
    }
}

fn hnorm(v: f32) -> f32 {
    let v = (0.5 * v / MAX_HEIGHT as f32).max(0.);
    if v > 1. {
        0.
    } else {
        v
    }
}

fn max_pos(x: i32, z: i32) -> i32 {
    if x < 0 || z < 0 {
        0
    } else {
        x.max(z)
    }
}

fn values(x: i32, z: i32) -> (f32, f32, f32) {
    let halfh = MAX_HEIGHT as i32 / 2;
    let x = x + halfh;
    let z = z + halfh;
    let y = max_pos(x, z) as f32;
    let t = x as f32;
    let h = z as f32;
    (hnorm(y), hnorm(t), hnorm(h))
}

impl TerrainGen for DebugGen {
    fn new(seed: u32, config: std::collections::HashMap<String, f32>) -> Self
    where
        Self: Sized + Clone,
    {
        DebugGen {
            seed,
            config,
            soils: WeightedPoints::from_csv("assets/data/soils_condition.csv").unwrap(),
        }
    }

    fn gen(
        &self,
        col: (i32, i32),
    ) -> [Option<crate::chunk::Chunk>; crate::chunk_map::MAX_HEIGHT / crate::chunk::CHUNK_S1] {
        let mut res = array![_ => None; MAX_HEIGHT / chunk::CHUNK_S1];
        let cx = col.0 * CHUNK_S1;
        let cz = col.1 * CHUNK_S1;
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let (y, t, h) = values(cx + dx, cz + dz);
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
