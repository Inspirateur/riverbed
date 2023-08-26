use crate::{BlocPos, Blocs, Bloc};
const SIGNS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

pub trait Growable: Send + Sync {
    fn grow(&self, dist: f32, pos: BlocPos, world: &mut Blocs);
}


pub fn leaf_disk(world: &mut Blocs, center: BlocPos, dist: u32, leaf: Bloc) {
    let dist = dist as i32;
    for z in 0..dist {
        let max_x = ((dist.pow(2)-z.pow(2)) as f32).sqrt() as i32;
        for x in 0..max_x {
            for (sx, sz) in SIGNS {
                world.set_if_empty(BlocPos {
                    realm: center.realm, 
                    x: center.x + sx*x,
                    y: center.y,
                    z: center.z + sz*z
                }, leaf)
            }
        }
    }
}