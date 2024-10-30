mod utils;
mod counter;
mod closest;
pub mod ranges;
pub mod points;
use std::fmt::Debug;
pub use closest::*;
use crate::counter::Counter;


pub fn print_coverage<const D: usize, E: Clone + PartialEq + Debug>(imap: impl ClosestTrait<D, E>, step: f32) {
    let mut coverage = imap.coverage(step);
    // add missing values
    imap.values().into_iter().for_each(|value| if !coverage.iter().any(|(v, _)| *v == value) {
        coverage.push((value, 0.))
    });
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
    use std::ops::Range;
    use crate::{print_coverage, ranges, points};

    #[test]
    pub fn print_cov_ranges() {
        let imap: Vec<([Range<f32>; 4], String)> = ranges::from_csv("benches/plants_ranges.csv").unwrap();
        print_coverage(imap, 0.05);
    }

    #[test]
    pub fn print_cov_points() {
        let imap: Vec<([f32; 4], String)> = points::from_csv("benches/plants_points.csv").unwrap();
        print_coverage(imap, 0.05);
    }
}
