use csv::Error;
use itertools::zip;
use std::{cmp::Ordering, str::FromStr};

#[derive(Clone)]
struct WeightedPoint {
    values: Vec<f32>,
    weight: f32,
}

impl WeightedPoint {
    fn dist(&self, other: &WeightedPoint) -> f32 {
        let mut res = 0.;
        for (a, b) in zip(&self.values, &other.values) {
            res += (a - b).powi(2)
        }
        res.sqrt() / (self.weight * other.weight)
    }
}

pub struct WeightedPoints<E> {
    elems: Vec<E>,
    index: Vec<WeightedPoint>,
}

impl<E> WeightedPoints<E> {
    pub fn from_csv(path: &str) -> Result<Self, Error>
    where
        E: FromStr,
        <E as FromStr>::Err: std::fmt::Debug,
    {
        let mut reader = csv::Reader::from_path(path)?;
        let header = reader.headers()?;
        let is_weighted = header[header.len() - 1].eq("weight");
        let mut elems = Vec::new();
        let mut index = Vec::new();
        for record in reader.records() {
            let record = record?;
            elems.push(E::from_str(&record[0]).unwrap());
            let mut values: Vec<f32> = record
                .into_iter()
                .skip(1)
                .map(|s| s.trim().parse::<f32>().unwrap())
                .collect();
            if is_weighted {
                let weight = values.pop().unwrap();
                index.push(WeightedPoint { values, weight });
            } else {
                index.push(WeightedPoint { values, weight: 1. });
            }
        }
        Ok(WeightedPoints { elems, index })
    }

    // turn this into a interval tree if it's too slow
    pub fn closest<const D: usize>(&self, point: &[f32; D]) -> (E, f32)
    where
        E: Clone,
    {
        let point = WeightedPoint {
            values: point.to_vec(),
            weight: 1.,
        };
        let (i, dist) = self
            .index
            .iter()
            .map(|a| a.dist(&point))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();
        (self.elems[i].clone(), dist)
    }
}
