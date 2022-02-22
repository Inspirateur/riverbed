use itertools::izip;
use std::ops::Range;

pub struct FuzzyIndex<E: Copy, const D: usize> {
    data: Vec<(E, [(f32, f32); D])>,
}

impl<E: Copy, const D: usize> FuzzyIndex<E, D> {
    pub fn new() -> Self {
        FuzzyIndex { data: Vec::new() }
    }

    pub fn insert(&mut self, element: E, ranges: [Range<f32>; D]) {
        self.data.push((
            element,
            ranges.map(|range| {
                (
                    // center of the range
                    (range.end + range.start) / 2.,
                    // normalisation factor
                    1.9 / (range.end - range.start),
                )
            }),
        ));
    }

    pub fn closest(&self, point: &[f32; D]) -> Result<E, String> {
        if self.data.len() == 0 {
            return Err("No elements in the Index".to_string());
        }
        let mut max_score = f32::MIN;
        let mut max_e = &self.data[0].0;
        for (elem, ranges) in &self.data {
            let mut score = 1.;
            for (v, (c, h)) in izip!(point, ranges) {
                score *= (h - (c - v).abs() * h.powi(2)).max(0.);
            }
            if score > max_score {
                max_score = score;
                max_e = elem;
            }
        }
        assert!(max_score > 0.);
        Ok(*max_e)
    }

    pub fn sample(&self, point: [f32; D]) -> Option<E> {
        todo!()
    }
}
