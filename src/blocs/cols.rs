use std::collections::{HashMap, HashSet};
use std::vec::Drain;
use bevy::prelude::Resource;
use itertools::Itertools;
use crate::{ChunkedPos, ColedPos};
use crate::bloc::Bloc;
use super::pos::{ChunkPos2D, BlocPos, BlocPos2D};
use super::Col;

pub type Cols<E> = HashMap<ChunkPos2D, E>;


#[derive(Resource)]
pub struct Blocs {
    pub cols: Cols<Col>,
    tracking: HashSet<ChunkPos2D>,
    changes: Vec<(BlocPos, Bloc)>
}

impl Blocs {
    pub fn new() -> Self {
        Blocs {
            cols: Cols::new(),
            tracking: HashSet::new(),
            changes: Vec::new()
        }
    }

    pub fn set_bloc(&mut self, pos: BlocPos, bloc: Bloc) {
        let (colpos, coledpos) = pos.into();
        if self.tracking.contains(&colpos) {
            self.changes.push((pos, bloc));
        }
        self.cols.entry(colpos).or_insert(Col::new()).set(coledpos, bloc);
    }

    pub fn set_if_empty(&mut self, pos: BlocPos, bloc: Bloc) {
        let (colpos, coledpos) = pos.into();
        if self.tracking.contains(&colpos) {
            self.changes.push((pos, bloc));
        }
        self.cols.entry(colpos).or_insert(Col::new()).set_if_empty(coledpos, bloc);
    }
    
    pub fn get_block(&self, pos: BlocPos) -> Bloc {
        let (colpos, coledpos) = pos.into();
        match self.cols.get(&colpos) {
            None => Bloc::default(),
            Some(col) => col.get(coledpos)
        }
    }

    pub fn top_block(&self, pos: BlocPos2D) -> (Bloc, i32) {
        let (colpos, pos2d) = pos.into();
        match self.cols.get(&colpos) {
            None => (Bloc::default(), 0),
            Some(col) => col.top(pos2d)
        }
    }

    pub fn track(&mut self, col: ChunkPos2D) {
        self.tracking.insert(col);
    }

    pub fn untrack(&mut self, col: &ChunkPos2D) {
        self.tracking.remove(col);
    }

    pub fn pull_changes(&mut self) -> Vec<(ChunkPos2D, Vec<(ColedPos, Bloc)>)> {
        let mut res = Vec::new();
        for (key, grouped) in &self.changes.drain(..)
            .map(|(pos, bloc)| {
                let (colpos, coledpos) = <(ChunkPos2D, ColedPos)>::from(pos);
                (colpos, (coledpos, bloc))
            })
            .sorted_by_key(|(colpos, _)| *colpos)
            .group_by(|(colpos, _)| *colpos) {
            res.push((key, grouped.map(|(_, data)| data).collect_vec()));
        }
        res
    }
}