use crate::{biomes::Biome, coverage::CoverageTrait};
use std::{
    collections::{BTreeMap, HashMap},
    ops::Index,
    str::FromStr,
};
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString, Eq, Hash)]
pub enum BiomeParam {
    Continentalness,
    Mountainness,
    Temperature,
    Humidity,
    Trees,
    Ph,
}

pub struct BiomePoints<const D: usize> {
    pub parameters: [BiomeParam; D],
    pub points: Vec<([f32; D], Biome)>,
    indexes: BTreeMap<Biome, usize>,
}

impl<const D: usize> BiomePoints<D> {
    pub fn from_csv(path: &str) -> Self {
        let mut points = Vec::new();
        let mut reader = csv::Reader::from_path(path).unwrap();
        let header = reader.headers().unwrap();
        let parameters: [BiomeParam; D] = core::array::from_fn(|i| {
            let header = &header[i + 1];
            BiomeParam::from_str(header.trim()).unwrap_or_else(|_| {
                panic!(
                    "Failed to deserialize biome parameter from header '{}'",
                    header
                )
            })
        });
        for record in reader.records() {
            let record = record.unwrap();
            let Ok(elem) = Biome::from_str(&record[0]) else {
                panic!("Failed to deserialize value '{}'", &record[0]);
            };
            let intervals: [f32; D] =
                core::array::from_fn(|i| record[i + 1].trim().parse::<f32>().unwrap());
            points.push((intervals, elem));
        }
        Self {
            parameters,
            indexes: BTreeMap::from_iter(
                points.iter().enumerate().map(|(i, (_, biome))| (*biome, i)),
            ),
            points,
        }
    }

    /// Returns the biomes that are closest to the given parameters, within a threshold of normalized distance.
    pub fn closest_biomes(&self, params: [f32; D], threshold: f32) -> Vec<Biome> {
        self.points
            .iter()
            .filter_map(|(point, biome)| {
                if dist(point, &params) < threshold {
                    Some(*biome)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn dist_from(&self, params: &[f32; D], biome: &Biome) -> f32 {
        dist(&self.points[self.indexes[biome]].0, &params)
    }
}

impl<const D: usize> CoverageTrait<D, Biome> for BiomePoints<D> {
    fn closest(&self, point: [f32; D]) -> (&Biome, f32) {
        let mut candidates = self
            .points
            .iter()
            .map(|(b_point, value)| (value, dist(&point, b_point)));
        let mut res = candidates.next().unwrap();
        for (v, sign_dist) in candidates {
            if res.1 < sign_dist {
                res = (v, sign_dist);
            }
        }
        res
    }
}

fn dist<const D: usize>(a: &[f32; D], b: &[f32; D]) -> f32 {
    let mut res = 0.;
    for i in 0..D {
        res += (a[i] - b[i]).powi(2);
    }
    res.sqrt() / (D as f32).sqrt()
}

/// Every Vec must have the same length, which is the number of block columns in a chunk (CHUNK_S1 * CHUNK_S1).
pub struct BiomeParameters(pub HashMap<BiomeParam, Vec<f32>>);

impl BiomeParameters {
    pub fn view<const D: usize>(&self, params: [BiomeParam; D]) -> Vec<[f32; D]> {
        let len = self.0.iter().next().unwrap().1.len();
        let mut iters = params.map(|p| self[p].iter());

        (0..len)
            .map(|_| std::array::from_fn(|i| *iters[i].next().unwrap()))
            .collect()
    }

    fn avg(vec: &Vec<f32>) -> f32 {
        vec.iter().fold(0., |i, a| i + *a) / (vec.len() as f32)
    }

    pub fn average<const D: usize>(&self, params: [BiomeParam; D]) -> [f32; D] {
        params.map(|p| {
            Self::avg(
                self.0
                    .get(&p)
                    .unwrap_or_else(|| panic!("Missing biome parameter '{p:?}'")),
            )
        })
    }
}

impl Index<BiomeParam> for BiomeParameters {
    type Output = Vec<f32>;

    fn index(&self, index: BiomeParam) -> &Self::Output {
        self.0
            .get(&index)
            .unwrap_or_else(|| panic!("Missing biome parameter '{index:?}'"))
    }
}
