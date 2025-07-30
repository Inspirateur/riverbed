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

    /// Returns the biomes that are closest to the given parameters, within a threshold of distance relative to the closest biome.
    /// eg: if the closest biome has a distance of 0.2 (in parameter space), and the relative threshold is set to 2, 
    /// then any biome with a distance greater than 0.2*2 = 0.4 will be rejected 
    pub fn closest_biomes(&self, params: [f32; D], relative_threshold: f32) -> Vec<Biome> {
        let dists: Vec<_> = self.points.iter().map(|(point, biome)| (dist(point, &params), biome)).collect();
        let min_dist = dists.iter().min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap()).unwrap().0;
        dists.into_iter().filter_map(|(dist, &biome)| if dist/min_dist < relative_threshold { Some(biome) } else { None }).collect()
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
    res.sqrt()
}

pub struct BiomeParameters {
    pub continentalness: Vec<f32>,
    pub temperature: Vec<f32>,
}

impl BiomeParameters {
    pub fn at(&self, dx: usize, dz: usize) -> [f32; 2] {
        let continentalness = self.continentalness[dz*CHUNK_S1 + dx];
        let temperature = self.temperature[dz*CHUNK_S1 + dx];
        [continentalness, temperature]
    }
}