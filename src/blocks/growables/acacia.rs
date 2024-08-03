use crate::world::{BlockPos, VoxelWorld};
use crate::blocks::Block;
use super::utils::leaf_disk;

pub fn grow_acacia(world: &VoxelWorld, pos: BlockPos, _seed: i32, dist: f32) {
    let height = 10-(dist*7.) as i32;
    let mut pos = pos;
    for _ in 0..height {
        world.set_block(pos, Block::AcaciaLog);
        pos.y += 1;
    }

    pos.y -= 1;
    leaf_disk(world, pos, 1, Block::AcaciaLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-3, Block::AcaciaLeaves);
    pos.y += 1;
    leaf_disk(world, pos, height as u32-4, Block::AcaciaLeaves);
    if height > 6 {
        pos.y += 1;
        leaf_disk(world, pos, height as u32-5, Block::AcaciaLeaves);
    }
}
