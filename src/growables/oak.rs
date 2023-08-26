use crate::{BlocPos, Blocs, Bloc};
use super::utils::leaf_disk;

pub fn grow_oak(world: &mut Blocs, pos: BlocPos, dist: f32) {
    let height = 6-(dist*2.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_bloc(pos, Bloc::OakWood);
        pos.y += 1;
    }
    pos.y -= 1;
    leaf_disk(world, pos, 1, Bloc::OakLeave);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-1, Bloc::OakLeave);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-3, Bloc::OakLeave);
}
