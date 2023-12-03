use crate::blocs::{BlocPos, Blocs, Bloc};
use super::utils::leaf_disk;

pub fn grow_oak(world: &mut Blocs, pos: BlocPos, seed: i32, dist: f32) {
    let height = 10-(dist*7.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_bloc(pos, Bloc::OakLog);
        pos.y += 1;
    }

    pos.y -= 1;
    leaf_disk(world, pos, 1, Bloc::OakLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-3, Bloc::OakLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-4, Bloc::OakLeaves);
    if height > 6 {
        pos.y += 1;
        leaf_disk(world, pos, height as u32-5, Bloc::OakLeaves);
    }
}
