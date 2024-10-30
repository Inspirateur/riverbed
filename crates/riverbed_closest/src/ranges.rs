use std::{ops::Range, str::FromStr};
use itertools::Itertools;
use anyhow::{Result, bail};
use crate::{closest::ClosestTrait, utils::{range_from_str, RangesUtil}};


impl<const D: usize, E: Clone> ClosestTrait<D, E> for Vec<([Range<f32>; D], E)> {
    fn closest(&self, point: [f32; D]) -> (&E, f32) {
        let mut candidates = self.iter()
            .map(|(ranges, value)| (value, ranges.sign_dist(&point)));
        let mut res = candidates.next().unwrap();
        for (v, sign_dist) in candidates {
            if res.1 < sign_dist {
                res = (v, sign_dist);
            }
        }
        res
    }

    fn values(&self) -> Vec<&E> {
        self.iter().map(|(_, value)| value).collect_vec()
    }
}

pub fn from_csv<const D: usize, E: FromStr>(path: &str) -> Result<Vec<([Range<f32>; D], E)>> {
    let mut res = Vec::new();
    let mut reader = csv::Reader::from_path(path)?;
    for record in reader.records() {
        let record = record?;
        let Ok(elem) = E::from_str(&record[0]) else {
            bail!("Failed to deserialize value '{}'", &record[0]);
        };
        let intervals: [Range<f32>; D] = core::array::from_fn(|i| range_from_str(&record[i+1]).unwrap());
        res.push((intervals, elem));
    }
    Ok(res)
}