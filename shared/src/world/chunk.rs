use packed_uints::PackedUints;
use crate::{block::Face, Block};
use super::{pos::{ChunkedPos, ColedPos}, utils::Palette};

#[derive(Debug)]
pub struct Chunk {
    pub data: PackedUints,
    pub palette: Palette<Block>,
}

pub trait ChunkTrait {
    fn get(&self, pos: ChunkedPos) -> &Block;
    fn set(&mut self, pos: ChunkedPos, block: Block);
    fn set_unpadded(&mut self, pos: ChunkedPos, block: Block);
    fn set_yrange(&mut self, pos: ChunkedPos, height: usize, block: Block);
    fn top(&self, pos: ColedPos) -> (&Block, usize);
    fn set_if_empty(&mut self, pos: ChunkedPos, block: Block) -> bool;
    fn copy_side_from(&mut self, other: &Self, face: Face);
}
