use std::ops::Range;
use itertools::Itertools;
use crate::counter::Counter;

pub trait ClosestTrait<const D: usize, E: Clone> {
    /// Returns the closest object from the point and a matching score in ]-inf; 1]. 
    /// A matching score of 1 means exact match; negative values mean that the object is not "suitable".
    /// May panic if the collection is empty
    fn closest(&self, point: [f32; D]) -> (&E, f32);

    fn values(&self) -> Vec<&E>;

    /// Estimates the proportion space for which a non negative value is returned (ie covered space)
    fn coverage(&self, step: f32) -> Vec<(&E, f32)>
        where E: PartialEq<E> 
    {
        let mut res = Vec::new();
        let samples = core::array::from_fn::<Range<f32>, D, _>(|_| 0f32..1f32).into_iter().map(
            |range| {
                let len = ((range.end-range.start)/step) as u32;
                (0..=len).map(move |i| range.start + i as f32*step)
            }
        ).multi_cartesian_product();
        let mut count = 0;
        for point in samples {
            let (value, dist) = self.closest(point.try_into().unwrap());
            if dist >= 0. {
                res.add(value);
            }
            count += 1;
        }
        res.divide(count as f32);
        res
    }
}