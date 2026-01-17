mod face;
mod block;

// Generated code from blocks.def
include!(concat!(env!("OUT_DIR"), "/blocks.rs"));

pub use face::*;
