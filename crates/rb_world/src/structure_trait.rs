use rb_pos::BlockPos;

use crate::VoxelWorld;

/// A trait to handle generating structures.
pub trait StructureTrait {
    fn origin(&self) -> BlockPos;
    /// Returns the maximum width of the structure in blocks.
    /// This is used to determine how many columns must be loaded around the position before generating the structure.
    fn max_block_width(&self) -> i32;
    /// Grow the structure in the world with the given seed.
    /// It is assumed that all the necessary chunks are already loaded.
    fn grow(&self, world: &VoxelWorld, seed: i32);
}
