use rb_pos::{BlockPos, CHUNK_S1, chunked};

use crate::VoxelWorld;

/// A trait to handle generating structures.
pub trait StructureTrait: Send + Sync {
    fn origin(&self) -> BlockPos;
    /// Returns the maximum block distance (Chebyshev) of the structure from its origin, on the X, Z plane
    /// (Y doesn't matter because generation is done with columns).
    ///
    /// Used to determine how many columns must be loaded around the position before generating the structure.
    fn max_block_dist(&self) -> usize;

    /// Returns the required loaded column distance (Chebyshev) required for the structure to grow.
    ///
    /// Anything bigger than 0 will have a required load distance of at least 1
    /// (because if it grows on the border of a column it'll need neighboring columns to be loaded)
    fn required_load_distance(&self) -> usize {
        let (q, r) = chunked::<CHUNK_S1, 1>(self.max_block_dist() as i32);
        q as usize + if r > 0 { 1 } else { 0 }
    }

    /// Grow the structure in the world with the given seed.
    /// It is assumed that all the necessary chunks are already loaded.
    fn grow(&self, world: &VoxelWorld, seed: u64);
}
