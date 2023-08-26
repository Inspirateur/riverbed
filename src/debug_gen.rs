use crate::{
    terrain_gen::TerrainGen,
    earth_gen::WATER_R
};
use ourcraft::{
    MAX_HEIGHT, CHUNK_S1,
    Bloc, Soils, Col, unchunked, Blocs, ChunkPos2D
};
use itertools::iproduct;
use nd_interval::NdInterval;
use std::{collections::HashMap, path::Path};

pub struct DebugGen {
    seed: u32,
    config: HashMap<String, f32>,
    soils: Soils,
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

impl DebugGen {
    fn new(seed: u32, config: std::collections::HashMap<String, f32>) -> Self
    where
        Self: Sized + Clone,
    {
        DebugGen {
            seed,
            config,
            soils: Soils::from_csv(Path::new("assets/data/soils_condition.csv")).unwrap(),
        }
    }
}

impl TerrainGen for DebugGen {
    fn gen(&self, world: &mut Blocs, pos: ChunkPos2D) {
        let mut col = Col::new();
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let (x, z) = (unchunked(pos.x, dx), unchunked(pos.z, dz));
            let (y, t, h) = values(x, z);
            let y = (WATER_R as f32*(y+1.) * MAX_HEIGHT as f32) as i32;
            assert!(y >= 0);
            col.set((dx, y, dz), *self.soils.closest([t as f32, h as f32]).unwrap_or((&Bloc::Dirt, 0.)).0);
            for y_ in (y-3)..y {
                if y_ < 0 {
                    break;
                }
                col.set((dx, y_, dz), Bloc::Dirt);
            }
        }
        // this is a bit too slow so we don't bother with it for now
        // col.fill_up(Bloc::Stone);
        world.0.insert(pos, col);
    }

    fn set_config(&mut self, config: HashMap<String, f32>) {
        todo!()
    }

    fn set_seed(&mut self, seed: u32) {
        todo!()
    }
}
