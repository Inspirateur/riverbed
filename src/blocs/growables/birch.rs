use crate::blocs::{BlocPos, Blocs, Bloc};
use super::utils::leaf_disk;

pub fn grow_birch(world: &mut Blocs, pos: BlocPos, _seed: i32, dist: f32) {
    let height = 7-(dist*3.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_bloc(pos, Bloc::BirchLog, false);
        pos.y += 1;
    }
    pos.y -= height/2;
    for i in 0..height {
        leaf_disk(world, pos, (1+(i).min(height-i)) as u32/2, Bloc::BirchLeaves);
        pos.y += 1;
    }
    world.set_bloc(pos, Bloc::BirchLeaves, false);
}
