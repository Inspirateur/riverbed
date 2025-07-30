mod coverage;
mod counter;
use std::fmt::Debug;
pub use coverage::CoverageTrait;

use crate::generation::coverage::counter::Counter;

pub fn print_coverage<const D: usize, E: Clone + PartialEq + Debug>(imap: impl CoverageTrait<D, E>, step: f32) {
    let mut coverage = imap.coverage(step);
    // .sum() refuses to work here for some reason
    let unassigned = 1f32 - coverage.iter().map(|(_, count)| *count).fold(0., |a, b| a + b);
    coverage.ordered();
    for (value, count) in coverage {
        println!("{:?}: {:.1}%", value, count*100.);
    }
    println!("---");
    println!("Empty: {:.1}%", unassigned*100.);
}

#[cfg(test)]
mod tests {
    use crate::generation::{biome_params::BiomePoints, coverage::print_coverage, plant_params::PlantRanges};

    #[test]
    pub fn print_plant_coverage() {
        let plants: PlantRanges<3> = PlantRanges::from_csv("assets/gen/plants_condition.csv");
        print_coverage(plants, 0.05);
    }

    #[test]
    pub fn print_biome_coverage() {
        let biomes: BiomePoints<2> = BiomePoints::from_csv("assets/gen/biomes.csv");
        print_coverage(biomes, 0.05);
    }
}
