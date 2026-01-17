//! Trait for block access and raycasting operations.
//!
//! This module provides a common interface for accessing blocks in a voxel world,
//! shared between client and server implementations.

use bevy::prelude::Vec3;

use crate::block::Block;
use crate::world::pos::pos3d::BlockPos;
use crate::world::realm::Realm;
use crate::world::BlockRayCastHit;

/// Trait for types that provide block access in a voxel world.
///
/// This trait abstracts the common operations needed for physics, raycasting,
/// and other world queries that both client and server need to perform.
pub trait BlockAccess {
    /// Get a block at the given position, returning Air for out-of-bounds or unloaded chunks.
    fn get_block_safe(&self, pos: BlockPos) -> Block;

    /// Perform a raycast against the world, finding the first targetable block.
    ///
    /// Uses a 3D DDA (Digital Differential Analyzer) algorithm to step through
    /// voxels along the ray until a targetable block is hit or the maximum
    /// distance is reached.
    ///
    /// # Arguments
    /// * `realm` - The realm to raycast in
    /// * `start` - The starting position of the ray
    /// * `dir` - The direction of the ray (should be normalized)
    /// * `dist` - The maximum distance to check
    ///
    /// # Returns
    /// `Some(BlockRayCastHit)` if a targetable block was hit, containing the
    /// block position and the face normal. `None` if no block was hit within
    /// the distance.
    fn raycast(
        &self,
        realm: Realm,
        start: Vec3,
        dir: Vec3,
        dist: f32,
    ) -> Option<BlockRayCastHit> {
        let mut pos = BlockPos {
            realm,
            x: start.x.floor() as i32,
            y: start.y.floor() as i32,
            z: start.z.floor() as i32,
        };
        let mut last_pos;
        let sx = dir.x.signum() as i32;
        let sy = dir.y.signum() as i32;
        let sz = dir.z.signum() as i32;
        if sx == 0 && sy == 0 && sz == 0 {
            return None;
        }
        let next_x = (pos.x + sx.max(0)) as f32;
        let next_y = (pos.y + sy.max(0)) as f32;
        let next_z = (pos.z + sz.max(0)) as f32;
        let mut t_max_x = (next_x - start.x) / dir.x;
        let mut t_max_y = (next_y - start.y) / dir.y;
        let mut t_max_z = (next_z - start.z) / dir.z;
        let slope_x = 1. / dir.x.abs();
        let slope_y = 1. / dir.y.abs();
        let slope_z = 1. / dir.z.abs();
        loop {
            last_pos = pos;
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x >= dist {
                        return None;
                    };
                    pos.x += sx;
                    t_max_x += slope_x;
                } else {
                    if t_max_z >= dist {
                        return None;
                    };
                    pos.z += sz;
                    t_max_z += slope_z;
                }
            } else if t_max_y < t_max_z {
                if t_max_y >= dist {
                    return None;
                };
                pos.y += sy;
                t_max_y += slope_y;
            } else {
                if t_max_z >= dist {
                    return None;
                };
                pos.z += sz;
                t_max_z += slope_z;
            }
            if self.get_block_safe(pos).is_targetable() {
                return Some(BlockRayCastHit {
                    pos,
                    normal: Vec3 {
                        x: (last_pos.x - pos.x) as f32,
                        y: (last_pos.y - pos.y) as f32,
                        z: (last_pos.z - pos.z) as f32,
                    },
                });
            }
        }
    }
}
