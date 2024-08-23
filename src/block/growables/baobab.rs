use crate::world::{BlockPos, VoxelWorld};
use crate::block::Block;
use super::utils::leaf_disk;
const DIRS: [(i32, i32); 8] = [(-1, 1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

fn baobab_leaves(world: &VoxelWorld, pos: BlockPos, dir_x: i32, dir_z: i32, size: usize) {
    let pos = pos + (if dir_x == 1 {2} else {-1}, 0, if dir_z == 1 {2} else {-1});
    world.set_block(pos, Block::AcaciaLog);
    leaf_disk(world, pos + (0, -1, 0), 1, Block::AcaciaLeaves);
    leaf_disk(world, pos + (dir_x, 0, dir_z), size as u32, Block::AcaciaLeaves);
}

pub fn grow_baobab(world: &VoxelWorld, pos: BlockPos, seed: i32, dist: f32) {
    let height = 30-(dist*6.) as i32;
    let mut pos = pos;
    let rng = pos.prng(seed);
    for i in 0..height {
        if i >= height/3 && i & 0b1 == 0 {
            let (dir_x, dir_z) = DIRS[((i as usize/2)^rng) & 0b111];
            baobab_leaves(
                world, pos, 
                dir_x, 
                dir_z,
                (i/3) as usize
            );
            if i >= 2*height/3 {
                baobab_leaves(
                    world, pos, 
                    -dir_x, 
                    -dir_z,
                    (i/3) as usize
                );
            }
        }
        world.set_block(pos, Block::AcaciaLog);
        world.set_block(pos + (1, 0, 0), Block::AcaciaLog);
        world.set_block(pos + (0, 0, 1), Block::AcaciaLog);
        world.set_block(pos + (1, 0, 1), Block::AcaciaLog);
        pos.y += 1;
    }
    world.set_block(pos, Block::SpruceLeaves);
}
