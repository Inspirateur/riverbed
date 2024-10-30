use std::ops::Range;
use anyhow::{Result, anyhow};

pub(crate) trait RangeUtil {
    fn sign_dist(&self, p: f32) -> f32;
}

impl RangeUtil for Range<f32> {
    fn sign_dist(&self, p: f32) -> f32 {
        2.*(p-self.start).min(self.end-p)/(self.end-self.start)
    }
}

pub(crate) trait RangesUtil<const D: usize> {
    fn sign_dist(&self, point: &[f32; D]) -> f32;
}

impl<const D: usize> RangesUtil<D> for [Range<f32>; D] {
    fn sign_dist(&self, point: &[f32; D]) -> f32 {
        self.into_iter().zip(point)
            .map(|(range, p)| range.sign_dist(*p))
            .fold(f32::INFINITY, |a, b| a.min(b))
    }
}


pub(crate) fn range_from_str(str: &str) -> Result<Range<f32>> {
    let (start, end) = str.trim().split_once(";").ok_or(
        anyhow!("expect format start;end, got '{}'", str)
    )?;
    let start = start.trim().parse::<f32>()?;
    let end = end.trim().parse::<f32>()?;
    Ok(start..end)
}