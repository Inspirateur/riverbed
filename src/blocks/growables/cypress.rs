use crate::blocks::{BlockPos, Blocks, Block};
use super::utils::leaf_disk;

pub fn grow_cypress(world: &Blocks, pos: BlockPos, _seed: i32, dist: f32) {
    let height = 11-(dist*3.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_block(pos, Block::SpruceLog);
        pos.y += 1;
    }
    pos.y -= height/2;
    for i in 0..height {
        leaf_disk(world, pos, (1+(i).min(height-i)) as u32/2, Block::SpruceLeaves);
        pos.y += 1;
    }
    world.set_block(pos, Block::SpruceLeaves);
}
