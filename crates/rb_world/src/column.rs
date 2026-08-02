use rb_block::Block;
use rb_pos::{CHUNK_S1, ChunkedPos, ChunkedPos2d, chunked, unchunked};

use crate::{Chunk, Y_CHUNKS};

#[derive(Debug, Default)]
pub struct Column(pub [Chunk; Y_CHUNKS]);

impl Column {
    pub fn set_yrange(&mut self, dx: usize, dz: usize, top: i32, mut height: usize, block: Block) {
        let (mut cy, mut dy) = chunked::<{ CHUNK_S1 }, 1>(top);
        while height > 0 && cy >= 0 {
            let h = height.min(dy);
            self.0[cy as usize].set_yrange(
                ChunkedPos {
                    x: dx,
                    y: dy,
                    z: dz,
                },
                h,
                block,
            );
            height -= h;
            cy -= 1;
            dy = CHUNK_S1 - 1;
        }
    }

    pub fn top_block(&self, in_col_pos: ChunkedPos2d) -> (Block, i32) {
        for y in (0..Y_CHUNKS).rev() {
            let (&block, block_y) = self.0[y].top(in_col_pos);
            if block != Block::Air {
                return (block.clone(), unchunked::<CHUNK_S1, 1>(y as i32, block_y));
            }
        }
        (Block::Air, 0)
    }
}
