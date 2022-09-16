use std::collections::HashMap;
use crate::pos::{Realm, ChunkPos2D, BlocPos, BlocPos2D};
use crate::blocs::{Col, Bloc};
use strum::{EnumCount, IntoEnumIterator};
use array_macro::array;
pub type Blocs = Cols<Col>;

pub struct Cols<E> {
    cols: [HashMap<(i32, i32), E>; Realm::COUNT],
}

impl<E> Cols<E> {
    pub fn new() -> Self {
        Cols { 
            cols: array![_ => HashMap::new(); Realm::COUNT],
        }
    }

    pub fn get_col(&self, col: ChunkPos2D) -> Option<&E> {
        self.cols[col.realm as usize].get(&(col.x, col.z))
    }

    pub fn insert_col(&mut self, pos: ChunkPos2D, col: E) {
        self.cols[pos.realm as usize].insert((pos.x, pos.z), col);
    }

    pub fn remove_col(&mut self, pos: ChunkPos2D) -> bool {
        self.cols[pos.realm as usize]
            .remove(&(pos.x, pos.z))
            .is_some()
    }

    pub fn contains_col(&self, pos: ChunkPos2D) -> bool {
        self.cols[pos.realm as usize].contains_key(&(pos.x, pos.z))
    }

    pub fn extend(&mut self, other: Cols<E>) {
        for (i, chunks) in other.cols.into_iter().enumerate() {
            self.cols[i].extend(chunks);
        }
    }

    pub fn cols(
        &self,
    ) -> impl Iterator<Item = (ChunkPos2D, &E)> {
        Realm::iter().flat_map(|realm| {
            self.cols[realm as usize].iter().map(move |((x, z), c)| {
                (
                    ChunkPos2D {
                        realm,
                        x: *x,
                        z: *z,
                    },
                    c,
                )
            })
        })
    }
}

impl Cols<Col> {
    pub fn get(&self, pos: BlocPos) -> Bloc {
        let (colpos, coledpos) = pos.into();
        self.get_col(colpos).unwrap().get(coledpos)
    }

    pub fn top(&self, pos: BlocPos2D) -> (Bloc, i32) {
        let (colpos, pos2d) = pos.into();
        self.get_col(colpos).unwrap().top(pos2d)
    }
}
