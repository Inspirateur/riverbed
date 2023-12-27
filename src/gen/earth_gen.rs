use crate::gen::terrain_gen::TerrainGen;
use bevy::prelude::info_span;
use crate::blocs::{CHUNK_S1, ColPos, Blocs, BlocPos2d, BlocPos, CHUNK_S1I};
use crate::blocs::{MAX_GEN_HEIGHT, Bloc, Soils, Trees};
use noise_algebra::NoiseSource;
use itertools::iproduct;
use std::{collections::HashMap, path::Path, ops::RangeInclusive};
use nd_interval::NdInterval;
pub const WATER_R: f32 = 0.3;
pub const WATER_H: i32 = (crate::blocs::MAX_GEN_HEIGHT as f32*WATER_R) as i32-8;

pub struct Earth {
    soils: Soils,
    trees: Trees,
    seed: i32,
    config: HashMap<String, f32>,
}

fn pos_to_range(pos: ColPos) -> [RangeInclusive<i32>; 2] {
    let x = pos.z*CHUNK_S1I;
    let y = pos.x*CHUNK_S1I;
    [x..=(x+CHUNK_S1I-1), y..=(y+CHUNK_S1I-1)]
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
    fn gen(&self, world: &mut Blocs, col: ColPos) {
        let range = pos_to_range(col);
        let gen_span = info_span!("noise gen", name = "noise gen").entered();
        let mut n = NoiseSource::new(range, self.seed, 1);
        let landratio = self.config.get("land_ratio").copied().unwrap_or(0.4);
        let continentalness = (n.simplex(0.1) + n.simplex(0.3) * 0.3).normalize().pos();
        let base_land = continentalness.clone().threshold(1.-landratio) 
            + n.simplex(1.)*0.2 
            + n.simplex(2.)*0.1;
        let moutain = (n.simplex(0.3) + n.simplex(1.)*0.3)
        .normalize().pos().threshold(0.95)*base_land.clone();

        let ys = base_land*WATER_R + moutain.clone()*(1.-WATER_R);
        let ts = (n.simplex(0.1) + n.simplex(0.3)*0.3).normalize().pos();
        // closer to the ocean => more humidity
        // lower temp => less humidity
        let hs = (!ts.clone()*0.5 + !continentalness*0.5 + n.simplex(0.4).pos()).normalize();
        let ph = (n.simplex(0.5) + n.simplex(2.)*0.2).normalize().pos();
        // convert y to convenient values
        let ys = ys.map(|y| (y.clamp(0., 1.) * MAX_GEN_HEIGHT as f32) as i32);
        gen_span.exit();
        let fill_span = info_span!("chunk filling", name = "chunk filling").entered();
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let (y, t, h) = (ys[[dx, dz]], ts[[dx, dz]], hs[[dx, dz]]); 
            
            let bloc = if y <= WATER_H {
                Bloc::Sand
            } else {
                match self.soils.closest([t, h]) {
                    Some((bloc, _)) => *bloc,
                    None => Bloc::Dirt,
                }
            };
            if moutain[[dx, dz]] > 0.2 {
                world.set_yrange(col, (dx, dz), y, 1, bloc);
                world.set_yrange(col, (dx, dz), y-1, 4, Bloc::Stone);
            } else {
                world.set_yrange(col, (dx, dz), y, 3, bloc);
            }
            let water_height = WATER_H-y;
            if water_height > 0 {
                world.set_yrange(col, (dx, dz), WATER_H, water_height as usize, Bloc::SeaBlock);
            }
        }
        fill_span.exit();
        let tree_span = info_span!("tree gen", name = "tree gen").entered();
        let tree_spots = [(0, 0), (16, 0), (8, 16), (24, 16)];
        for spot in tree_spots {
            let rng = <BlocPos2d>::from((col, spot)).prng(self.seed);
            let dx = spot.0 + (rng & 0b111);
            let dz = spot.1 + ((rng >> 3) & 0b111);
            let h = (rng >> 5) & 0b11;
            let y = ys[[dx, dz]];
            let m = moutain[[dx, dz]];
            if y > WATER_H && (m < 0.1 || m > 0.7) {
                if let Some((tree, dist)) = self.trees.closest([
                    ts[[dx, dz]], 
                    hs[[dx, dz]], 
                    ph[[dx, dz]], 
                    y as f32/MAX_GEN_HEIGHT as f32
                ]) {
                    let pos = BlocPos {
                        x: col.x*CHUNK_S1I+dx as i32, y, z: col.z*CHUNK_S1I+dz as i32, realm: col.realm
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
