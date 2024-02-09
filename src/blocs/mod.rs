mod utils;
mod chunk;
mod chunk_shape;
mod blocs;
mod pos;
mod bloc;
mod realm;
pub mod growables;
pub use bloc::*;
pub use realm::*;
pub use blocs::*;
pub use chunk::*;
pub use chunk_shape::*;
pub use pos::*;
pub use utils::*;
use lazy_static::lazy_static;
pub const CHUNK_S1: usize = 32;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNK_S3: usize = CHUNK_S1.pow(3);
pub const CHUNK_PADDED_S1: usize = CHUNK_S1 + 2;
pub const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub const MAX_GEN_HEIGHT: usize = 400;
pub const MAX_HEIGHT: usize = 512;
pub const Y_CHUNKS: usize = MAX_HEIGHT/CHUNK_S1;

lazy_static! {
    pub static ref CHUNK_SHAPE: YFirstShape = YFirstShape::new();
}