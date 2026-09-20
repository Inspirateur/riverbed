use crate::{
    coverage::CoverageTrait,
    range_utils::{RangesUtil, range_from_str},
    tree::Tree,
};
use std::{ops::Range, path::Path, str::FromStr};

pub struct PlantRanges<const D: usize>(Vec<([Range<f32>; D], Tree)>);

impl<const D: usize> PlantRanges<D> {
    pub fn from_csv<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut res = Vec::new();
        let mut reader = csv::Reader::from_path(path).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            let Ok(elem) = Tree::from_str(&record[0]) else {
                panic!("Failed to deserialize value '{}'", &record[0]);
            };
            let intervals: [Range<f32>; D] =
                core::array::from_fn(|i| range_from_str(&record[i + 1]).unwrap());
            res.push((intervals, elem));
        }
        Self(res)
    }
}

impl<const D: usize> CoverageTrait<D, Tree> for PlantRanges<D> {
    fn closest(&self, point: [f32; D]) -> (&Tree, f32) {
        let mut candidates = self
            .0
            .iter()
            .map(|(ranges, value)| (value, ranges.sign_dist(&point)));
        let mut res = candidates.next().unwrap();
        for (v, sign_dist) in candidates {
            if res.1 < sign_dist {
                res = (v, sign_dist);
            }
        }
        res
    }
}
