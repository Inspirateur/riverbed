use crate::world::{BlockPos, VoxelWorld};
use crate::block::Block;

pub trait Growable: Send + Sync {
    fn grow(&self, dist: f32, pos: BlockPos, world: &VoxelWorld);
}

#[inline]
fn signed_comb(x: i32, z: i32) -> Vec<(i32, i32)> {
    match (x, z) {
        (0, 0) => vec![(0, 0)],
        (0, _) => vec![(0, z), (0, -z)],
        (_, 0) => vec![(x, 0), (-x, 0)],
        (_, _) => vec![(x, z), (-x, z), (x, -z), (-x, -z)]
    }
}

#[inline]
pub fn leaf_disk(world: &VoxelWorld, center: BlockPos, dist: u32, leaf: Block) {
    let dist = dist as i32;
    for z in 0..=dist {
        let max_x = ((dist.pow(2)-z.pow(2)) as f32).sqrt() as i32;
        for x in 0..=max_x {
            for (dx, dz) in signed_comb(x, z) {
                world.set_if_empty(BlockPos {
                    realm: center.realm, 
                    x: center.x + dx,
                    y: center.y,
                    z: center.z + dz
                }, leaf)
            }
        }
    }
}
