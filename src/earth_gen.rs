use crate::terrain_gen::TerrainGen;
use bevy::prelude::info_span;
use ourcraft::{MAX_GEN_HEIGHT, Bloc, CHUNK_S1, Soils, ChunkPos2D, Blocs, Trees, BlocPos2D, BlocPos};
use noise_algebra::NoiseSource;
use itertools::iproduct;
use std::{collections::HashMap, path::Path, ops::RangeInclusive};
use nd_interval::NdInterval;
pub const WATER_R: f64 = 0.3;
pub const WATER_H: i32 = (MAX_GEN_HEIGHT as f64*WATER_R) as i32;
pub const CHUNK_S1i: i32 = CHUNK_S1 as i32;

pub struct Earth {
    soils: Soils,
    trees: Trees,
    seed: i32,
    config: HashMap<String, f32>,
}

fn pos_to_range(pos: ChunkPos2D) -> [RangeInclusive<i32>; 2] {
    let x = pos.z*CHUNK_S1i;
    let y = pos.x*CHUNK_S1i;
    [x..=(x+CHUNK_S1i-1), y..=(y+CHUNK_S1i-1)]
}

impl Earth {
    pub fn new(seed: u32, config: HashMap<String, f32>) -> Self {
        Earth {
            soils: Soils::from_csv(Path::new("assets/data/soils_condition.csv")).unwrap(),
            trees: Trees::from_csv(Path::new("assets/data/trees_condition.csv")).unwrap(),
            seed: seed as i32,
            config
        }
    }
}

impl TerrainGen for Earth {
    fn gen(&self, world: &mut Blocs, col: ChunkPos2D) {
        let range = pos_to_range(col);
        let gen_span = info_span!("noise gen", name = "noise gen").entered();
        let mut n = NoiseSource::new(range, self.seed, 1);
        let landratio = self.config.get("land_ratio").copied().unwrap_or(0.4) as f64;
        let cont = (n.simplex(0.5) + n.simplex(2.) * 0.4).normalize();
        let land = (cont.clone() + n.simplex(9.)*0.1).normalize().mask(landratio);
        let ocean = !cont.pos();
        // WATER_R is used to level land just above water level
        let ys = 0.009 + land.clone()*WATER_R + land*n.simplex(1.).pos().powi(3)*0.4;
        // more attitude => less temperature
        let ts = (n.simplex(0.2) + n.simplex(0.6)*0.3).normalize().pos();
        // closer to the ocean => more humidity
        // lower temp => less humidity
        let hs = (!ts.clone()*0.5 + ocean + n.simplex(0.5).pos()).normalize();
        let ph = (n.simplex(0.3) + n.simplex(0.9)*0.2).normalize().pos();
        println!("y {:?} t {:?} h {:?} ph {:?}", ys.domain, ts.domain, hs.domain, ph.domain);
        // convert y to convenient values
        let ys = ys.map(|y| (y.clamp(0., 1.) * MAX_GEN_HEIGHT as f64) as i32);
        gen_span.exit();
        let fill_span = info_span!("chunk filling", name = "chunk filling").entered();
        for (i, (dx, dz)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
            let (y, t, h) = (ys[i], ts[i], hs[i]);
            /* 
            let bloc = match self.soils.closest([t as f32, h as f32]) {
                Some((bloc, _)) => *bloc,
                None => Bloc::Dirt,
            };*/
            world.set_yrange(col, (dx, dz), y, 5, Bloc::Dirt);
        }
        // this is a bit too slow so we don't bother with it for now
        // col.fill_up(Bloc::Stone);
        fill_span.exit();
        let tree_span = info_span!("tree gen", name = "tree gen").entered();
        let tree_spots = [(0, 0), (16, 0), (8, 16), (24, 16)];
        for spot in tree_spots {
            let rng = <BlocPos2D>::from((col, spot)).prng(self.seed);
            let dx = spot.0 + (rng & 0b111);
            let dz = spot.1 + ((rng >> 3) & 0b111);
            let h = ((rng >> 5) & 0b11);
            let i = dx*CHUNK_S1 + dz;
            let y = ys[i];
            if y >= WATER_H {
                if let Some((tree, dist)) = self.trees.closest([
                    ts[i] as f32, 
                    hs[i] as f32, 
                    ph[i] as f32, 
                    y as f32/MAX_GEN_HEIGHT as f32
                ]) {
                    let pos = BlocPos {
                        x: col.x*CHUNK_S1i+dx as i32, y, z: col.z*CHUNK_S1i+dz as i32, realm: col.realm
                    };
                    tree.grow(world, pos, self.seed, dist+h as f32/10.);
                }
            }
        }
        tree_span.exit();
    }

    fn set_config(&mut self, config: HashMap<String, f32>) {
        todo!()
    }

    fn set_seed(&mut self, seed: u32) {
        todo!()
    }
}
