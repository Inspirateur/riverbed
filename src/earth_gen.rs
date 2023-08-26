use crate::terrain_gen::TerrainGen;
use ourcraft::{MAX_HEIGHT, Bloc, CHUNK_S1, Soils, Col, ChunkPos2D, Blocs, Plants, grow_oak, Pos2D, Pos};
use noise_algebra::NoiseSource;
use itertools::iproduct;
use std::{collections::HashMap, path::Path, ops::RangeInclusive};
use nd_interval::NdInterval;
pub const WATER_R: f64 = 0.3;
pub const WATER_H: i32 = (MAX_HEIGHT as f64*WATER_R) as i32;
pub const CHUNK_S1i: i32 = CHUNK_S1 as i32;

pub struct Earth {
    soils: Soils,
    plants: Plants,
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
            plants: Plants::from_csv(Path::new("assets/data/plants_condition.csv")).unwrap(),
            seed: seed as i32,
            config
        }
    }
}

impl TerrainGen for Earth {
    fn gen(&self, world: &mut Blocs, pos: ChunkPos2D) {
        let mut col: &mut Col = world.0.entry(pos).or_insert(Col::new());
        let range = pos_to_range(pos);
        let mut n = NoiseSource::new(range, self.seed, 1);
        let landratio = self.config.get("land_ratio").copied().unwrap_or(0.35) as f64;
        let cont = (n.simplex(0.7) + n.simplex(3.) * 0.3).normalize();
        let land = cont.clone() + n.simplex(9.) * 0.1;
        let ocean = !(cont*0.5 + 0.5);
        let land = land.normalize().mask(landratio);
        let mount_mask = (n.simplex(1.) + n.simplex(2.)*0.3).normalize().mask(0.2)*land.clone();
        let mount = (!n.simplex(0.8).powi(2) + n.simplex(1.5).powi(2)*0.4).normalize() * mount_mask;
        // WATER_R is used to ensure land remains above water even if water level is raised
        let ys = (0.009 + land*WATER_R + mount*(1.-WATER_R)).normalize();
        // more attitude => less temperature
        let ts = !ys.clone().powi(3) * (n.simplex(0.2)*0.5 + 0.5 + n.simplex(0.6)*0.3).normalize();
        // closer to the ocean => more humidity
        // higher temp => more humidity
        let hs = (ocean + ts.clone().powf(0.5) * (n.simplex(0.5)*0.5 + 0.5)).normalize();
        // convert y to convenient values
        let ys = ys.map(|y| (y.clamp(0., 1.) * MAX_HEIGHT as f64) as i32);
        for (i, (dx, dz)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
            let (y, t, h) = (ys[i], ts[i], hs[i]);
            let bloc = match self.soils.closest([t as f32, h as f32]) {
                Some((bloc, _)) => *bloc,
                None => Bloc::Dirt,
            };
            col.set((dx, y, dz), bloc);
            for y_ in (y-5)..y {
                if y_ < 0 {
                    break;
                }
                col.set((dx, y_, dz), Bloc::Dirt);
            }
        }
        // this is a bit too slow so we don't bother with it for now
        // col.fill_up(Bloc::Stone);
        grow_oak(world, Pos { x: pos.x*CHUNK_S1i, y: ys[0], z: pos.z*CHUNK_S1i, realm: pos.realm}, 0.);
    }

    fn set_config(&mut self, config: HashMap<String, f32>) {
        todo!()
    }

    fn set_seed(&mut self, seed: u32) {
        todo!()
    }
}
