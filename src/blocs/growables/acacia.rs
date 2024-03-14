use crate::blocs::{BlocPos, Blocs, Bloc};
use super::utils::leaf_disk;

pub fn grow_acacia(world: &Blocs, pos: BlocPos, _seed: i32, dist: f32) {
    let height = 10-(dist*7.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_bloc(pos, Bloc::AcaciaLog);
        pos.y += 1;
    }

    pos.y -= 1;
    leaf_disk(world, pos, 1, Bloc::AcaciaLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-3, Bloc::AcaciaLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-4, Bloc::AcaciaLeaves);
    if height > 6 {
        pos.y += 1;
        leaf_disk(world, pos, height as u32-5, Bloc::AcaciaLeaves);
    }
}
