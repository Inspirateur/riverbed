use crate::{
    bloc::{Bloc, Soils},
    terrain_gen::TerrainGen,
    MAX_HEIGHT, CHUNK_S1, blocs::Col
};
use noise_algebra::NoiseSource;
use itertools::iproduct;
use std::{collections::HashMap, path::Path, ops::RangeInclusive};
use nd_interval::NdInterval;
pub const WATER_R: f64 = 0.3;
pub const WATER_H: i32 = (MAX_HEIGHT as f64*WATER_R) as i32;
pub const CHUNK_S1i: i32 = CHUNK_S1 as i32;

#[derive(Clone)]
pub struct Earth {
    soils: Soils,
    seed: i32,
    config: HashMap<String, f32>,
}

fn pos_to_range(col_pos: (i32, i32)) -> [RangeInclusive<i32>; 2] {
    let x = col_pos.1*CHUNK_S1i;
    let y = col_pos.0*CHUNK_S1i;
    [x..=(x+CHUNK_S1i-1), y..=(y+CHUNK_S1i-1)]
}

impl TerrainGen for Earth {
    fn new(seed: u32, config: HashMap<String, f32>) -> Self
    where
        Self: Sized,
    {
        Earth {
            soils: Soils::from_csv(Path::new("assets/data/soils_condition.csv")).unwrap(),
            seed: seed as i32,
            config
        }
    }

    fn gen(&self, col_pos: (i32, i32)) -> Col {
        let mut col = Col::new();
        let range = pos_to_range(col_pos);
        let mut n = NoiseSource::new(range, self.seed, 1);
        let landratio = self.config.get("land_ratio").copied().unwrap_or(0.45) as f64;
        let cont = (n.simplex(0.7) + n.simplex(3.) * 0.3).normalize();
        let land = cont.clone() + n.simplex(9.) * 0.1;
        let ocean = !(cont*0.5 + 0.5);
        let land = land.normalize().mask(landratio);
        let mount_mask = (n.simplex(1.) + n.simplex(2.)*0.3).normalize().mask(0.2)*land.clone();
        let mount = (!n.simplex(0.8).powi(2) + n.simplex(1.5).powi(2)*0.4).normalize() * mount_mask;
        // WATER_R is used to ensure land remains above water even if water level is raised
        let ys = 0.009 + land*WATER_R + mount*(1.-WATER_R);
        // more attitude => less temperature
        let ts = !ys.clone().powi(3) * (n.simplex(0.2)*0.5 + 0.5 + n.simplex(0.6)*0.3).normalize();
        // closer to the ocean => more humidity
        // higher temp => more humidity
        let hs = (ocean + ts.clone().powf(0.5) * (n.simplex(0.5)*0.5 + 0.5)).normalize();
        for (i, (dx, dz)) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1).enumerate() {
            let (y, t, h) = (ys[i], ts[i], hs[i]);
            let y = (y * MAX_HEIGHT as f64) as i32;
            assert!(y >= 0);
            let bloc = match self.soils.closest([t as f32, h as f32]) {
                Some((bloc, _)) => *bloc,
                None => Bloc::Dirt,
            };
            col.set((dx, y, dz), bloc);
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
