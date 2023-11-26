use crate::blocs::{BlocPos, Blocs, Bloc};
use super::utils::leaf_disk;

pub fn grow_spruce(world: &mut Blocs, pos: BlocPos, seed: i32, dist: f32) {
    let height = 11-(dist*4.) as i32;
    let mut pos = pos;
    for i in 0..height {
        if i >= 3 && i % 2 == height % 2 {
            leaf_disk(world, pos, ((height-i+2)/2) as u32, Bloc::SpruceLeaves)
        }
        world.set_bloc(pos, Bloc::SpruceLog);
        pos.y += 1;
    }
    leaf_disk(world, pos, 1, Bloc::SpruceLeaves);
    pos.y += 1;
    world.set_bloc(pos, Bloc::SpruceLeaves);
}
