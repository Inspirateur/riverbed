use std::collections::HashMap;
use crate::pos::{ChunkPos2D, BlocPos, BlocPos2D};
use crate::blocs::{Col, Bloc};
pub type Cols<E> = HashMap<ChunkPos2D, E>;
pub type Blocs = Cols<Col>;
pub trait BlocsTrait {
    fn get_block(&self, pos: BlocPos) -> Bloc;

    fn top_block(&self, pos: BlocPos2D) -> (Bloc, i32);
}

impl BlocsTrait for Blocs {
    fn get_block(&self, pos: BlocPos) -> Bloc {
        let (colpos, coledpos) = pos.into();
        match self.get(&colpos) {
            None => Bloc::Air,
            Some(col) => col.get(coledpos)
        }
    }

    fn top_block(&self, pos: BlocPos2D) -> (Bloc, i32) {
        let (colpos, pos2d) = pos.into();
        match self.get(&colpos) {
            None => (Bloc::Bedrock, 0),
            Some(col) => col.top(pos2d)
        }
    }
}