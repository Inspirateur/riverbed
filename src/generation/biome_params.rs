use std::{collections::BTreeMap, str::FromStr};
use crate::{generation::{biomes::Biome, coverage::CoverageTrait}, world::CHUNK_S1};

pub struct BiomePoints<const D: usize> {
    points: Vec<([f32; D], Biome)>,
    indexes: BTreeMap<Biome, usize>
}

impl<const D: usize> BiomePoints<D> {
    pub fn from_csv(path: &str) -> Self {
        let mut points = Vec::new();
        let mut reader = csv::Reader::from_path(path).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            let Ok(elem) = Biome::from_str(&record[0]) else {
                panic!("Failed to deserialize value '{}'", &record[0]);
            };
            let intervals: [f32; D] = core::array::from_fn(|i| record[i+1].trim().parse::<f32>().unwrap());
            points.push((intervals, elem));
        }
        Self {
            indexes: BTreeMap::from_iter(points.iter().enumerate().map(|(i, (_, biome))| (*biome, i))),
            points
        }
    }

    /// Returns the biomes that are closest to the given parameters, within a threshold of normalized distance.
    pub fn closest_biomes(&self, params: [f32; D], threshold: f32) -> Vec<Biome> {
        self.points.iter().filter_map(|(point, biome)| if dist(point, &params) < threshold { Some(*biome) } else { None }).collect()
    }

    pub fn dist_from(&self, params: &[f32; D], biome: &Biome) -> f32 {
        dist(&self.points[self.indexes[biome]].0, &params)
    }
}

impl<const D: usize> CoverageTrait<D, Biome> for BiomePoints<D> {
    fn closest(&self, point: [f32; D]) -> (&Biome, f32) {
                let mut candidates = self.points.iter()
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
        res += (a[i]-b[i]).powi(2);
    }
    res.sqrt()/(D as f32).sqrt()
}

pub struct BiomeParameters {
    pub continentalness: Vec<f32>,
    pub mountainness: Vec<f32>,
    pub temperature: Vec<f32>,
    pub humidity: Vec<f32>
}

impl BiomeParameters {
    pub fn at(&self, dx: usize, dz: usize) -> [f32; 4] {
        let continentalness = self.continentalness[dz*CHUNK_S1 + dx];
        let mountainness = self.mountainness[dz*CHUNK_S1 + dx];
        let temperature = self.temperature[dz*CHUNK_S1 + dx];
        let humidity = self.humidity[dz*CHUNK_S1 + dx];
        [continentalness, mountainness, temperature, humidity]
    }
}