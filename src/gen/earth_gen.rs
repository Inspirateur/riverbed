use bevy::prelude::info_span;
use crate::blocs::{CHUNK_S1, ColPos, Blocs, BlocPos2d, BlocPos, CHUNK_S1I};
use crate::blocs::{MAX_GEN_HEIGHT, WATER_H, Bloc, Soils, Trees};
use noise_algebra::NoiseSource;
use itertools::iproduct;
use std::{collections::HashMap, path::Path, ops::RangeInclusive};
use nd_interval::NdInterval;
pub const WATER_R: f32 = WATER_H as f32/MAX_GEN_HEIGHT as f32;
pub const CONT_R: f32 = (WATER_H+2) as f32/MAX_GEN_HEIGHT as f32;
pub const CONT_COMPL: f32 = 1.-CONT_R;

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

    pub fn gen(&self, world: &Blocs, col: ColPos) {
        let landratio = self.config.get("land_ratio").copied().unwrap_or(0.4);
        let range = pos_to_range(col);
        // let gen_span = info_span!("noise gen", name = "noise gen").entered();
        let mut n = NoiseSource::new(range, self.seed, 1);
        let continentalness = n.simplex(0.2);
        let cont = (n.simplex(1.)*0.3 + n.simplex(5.)*0.1 + n.simplex(20.)*0.05 + &continentalness).normalize().cap(CONT_R);
        let rockyness = n.simplex(0.2);
        let rocks = !(n.simplex(2.)*0.3 + n.simplex(10.)*0.1 + n.simplex(100.)*0.04 + &rockyness).normalize().cap(0.06);

        let ts = (n.simplex(0.1) + n.simplex(0.4)*0.15 + n.simplex(8.)*0.06 + n.simplex(100.)*0.015).normalize();
        
        let roughness = (n.simplex(1.0)*0.2 + !ts.powi(2) + &rockyness).normalize().powi(2);
        let mountain = (n.ridge(0.3).powi(2) + n.simplex(3.)*0.1 + n.simplex(30.)*0.005).normalize()*&roughness;
        
        let hs = (n.simplex(0.1) + !continentalness*0.5 + n.simplex(10.)*0.1 + n.simplex(60.)*0.04).normalize();
        let ph = (n.simplex(1.) + n.simplex(4.)*0.2 + n.simplex(40.)*0.1).normalize();
        let trees = (n.simplex(2.) + &hs*0.3 + n.simplex(10.)*0.4 + n.simplex(40.)*0.2).normalize();
        let ys = cont + mountain*CONT_COMPL + &rocks;
        // convert y to convenient values
        let ys = ys.map(|y| (y * MAX_GEN_HEIGHT as f32) as i32);
        // gen_span.exit();
        // let fill_span = info_span!("chunk filling", name = "chunk filling").entered();
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let (y, t, h, rock) = (ys[[dx, dz]], ts[[dx, dz]], hs[[dx, dz]], rocks[[dx, dz]]); 
            let bloc = if rock > 0.001 {
                Bloc::Stone
            } else if y <= WATER_H {
                Bloc::Sand
            } else {
                match self.soils.closest([t, h]) {
                    Some((bloc, _)) => *bloc,
                    None => Bloc::Dirt,
                }
            };
            world.set_yrange(col, (dx, dz), y, 8, bloc);
            let water_height = WATER_H-y;
            if water_height > 0 {
                world.set_yrange(col, (dx, dz), WATER_H, water_height as usize, Bloc::SeaBlock);
            }
        }
        // fill_span.exit();
        let tree_span = info_span!("tree gen", name = "tree gen").entered();
        let tree_spots = [(0, 0), (16, 0), (8, 16), (24, 16)];
        for spot in tree_spots {
            let rng = <BlocPos2d>::from((col, spot)).prng(self.seed);
            let dx = spot.0 + (rng & 0b111);
            let dz = spot.1 + ((rng >> 3) & 0b111);
            let tree = trees[[dx, dz]];
            if tree < 0.5 { continue; }
            let h = (rng >> 5) & 0b11;
            let y = ys[[dx, dz]];
            if y > WATER_H {
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
}
