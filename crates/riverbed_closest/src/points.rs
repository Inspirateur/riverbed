use std::str::FromStr;
use crate::{ClosestResult, ClosestTrait};
use anyhow::{Result, bail};
use itertools::Itertools;

trait PointDistSq {
    fn dist(&self, other: &Self) -> f32;
}

impl<const D: usize> PointDistSq for [f32; D] {
    fn dist(&self, other: &Self) -> f32 {
        let mut res = 0.;
        for i in 0..D {
            res += (self[i]-other[i]).powi(2);
        }
        res
    }
}

impl<const D: usize, E: Clone> ClosestTrait<D, E> for Vec<([f32; D], E)> {
    fn closest(&self, point: [f32; D]) -> ClosestResult<E> {
        let mut candidates = self.iter()
            .map(|(points, value)| (value, points.dist(&point)));
        let mut closest1 = candidates.next().unwrap();
        let Some(mut closest2) = candidates.next() else {
            return ClosestResult {
                closest: closest1.0,
                score: 1.,
                next_closest: None,
            }
        };
        if closest2.1 < closest1.1 {
            (closest1, closest2) = (closest2, closest1);
        }
        for (v, dist) in candidates {
            if dist < closest1.1 {
                closest2 = closest1;
                closest1 = (v, dist);
            } else if dist < closest2.1 {
                closest2 = (v, dist);
            }
        }
        ClosestResult {
            closest: closest1.0,
            score: 1. - 2. * closest1.1 / (closest1.1 + closest2.1),
            next_closest: Some(closest2.0),
        }
    }

    fn values(&self) -> Vec<&E> {
        self.iter().map(|(_, value)| value).collect_vec()
    }
}

pub fn from_csv<const D: usize, E: FromStr>(path: &str) -> Result<Vec<([f32; D], E)>> {
    let mut res = Vec::new();
    let mut reader = csv::Reader::from_path(path)?;
    for record in reader.records() {
        let record = record?;
        let Ok(elem) = E::from_str(&record[0]) else {
            bail!("Failed to deserialize value '{}'", &record[0]);
        };
        let intervals: [f32; D] = core::array::from_fn(|i| record[i+1].trim().parse::<f32>().unwrap());
        res.push((intervals, elem));
    }
    Ok(res)
}