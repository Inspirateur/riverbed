use crate::blocks::{BlockPos, Blocks, Block};
use super::utils::leaf_disk;

pub fn grow_oak(world: &Blocks, pos: BlockPos, _seed: i32, dist: f32) {
    let height = 12-(dist*7.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_block(pos, Block::OakLog);
        pos.y += 1;
    }

    pos.y -= 1;
    leaf_disk(world, pos, 1, Block::OakLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-3, Block::OakLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-4, Block::OakLeaves);
    if height > 6 {
        pos.y += 1;
        leaf_disk(world, pos, height as u32-5, Block::OakLeaves);
    }
}
