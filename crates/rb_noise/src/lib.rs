use num_traits::MulAddAssign;
use quick_noise::{Fbm, Grid, Perlin};
// manual scaling required because library oversight
// https://github.com/verpeteren/rust-simd-noise/issues/23
const S_FBM: f32 = 0.7;
const S_RIDGE: f32 = 1.4;
const S_FREQ: f32 = 0.1;
/// 2D FBM with 5 octaves in [0;1]
pub fn fbm(
    chunk_x: i32,
    width: usize,
    chunk_z: i32,
    height: usize,
    seed: u32,
    freq: f32,
) -> Vec<f32> {
    let mut res = Grid::<2>::new(width, height)
        .grid_position(chunk_x, chunk_z)
        .seed(seed as i64)
        .builder::<Fbm, Perlin>()
        .seed(seed as i64)
        .octaves(5)
        .frequency(freq * S_FREQ)
        .finalize(true)
        .build();
    res.iter_mut().for_each(|v| v.mul_add_assign(S_FBM, 0.5));
    res
}

/// 2D FBM with 5 octaves in [min;max]
pub fn fbm_scaled(
    chunk_x: i32,
    width: usize,
    chunk_z: i32,
    height: usize,
    seed: u32,
    freq: f32,
    min: f32,
    max: f32,
) -> Vec<f32> {
    let delta = max - min;
    let s = S_FBM * delta;
    let c = 0.5 * delta + min;
    let mut res = Grid::<2>::new(width, height)
        .grid_position(chunk_x, chunk_z)
        .seed(seed as i64)
        .builder::<Fbm, Perlin>()
        .seed(seed as i64)
        .octaves(5)
        .frequency(freq * S_FREQ)
        .finalize(true)
        .build();
    res.iter_mut().for_each(|v| v.mul_add_assign(s, c));
    res
}

/// 2D Ridge noise in [0;1]
pub fn ridge(
    chunk_x: i32,
    width: usize,
    chunk_z: i32,
    height: usize,
    seed: u32,
    freq: f32,
) -> Vec<f32> {
    let mut res = Grid::<2>::new(width, height)
        .grid_position(chunk_x, chunk_z)
        .seed(seed as i64)
        .builder::<Fbm, Perlin>()
        .seed(seed as i64)
        .octaves(5)
        .frequency(freq * S_FREQ)
        .finalize(true)
        .build();
    res.iter_mut().for_each(|v| *v = (*v * S_RIDGE).abs());
    res
}

/// 2D Ridge noise in [min;max]
pub fn ridge_scaled(
    chunk_x: i32,
    width: usize,
    chunk_z: i32,
    height: usize,
    seed: u32,
    freq: f32,
    min: f32,
    max: f32,
) -> Vec<f32> {
    let delta = max - min;
    let s = S_RIDGE * delta;
    let mut res = Grid::<2>::new(width, height)
        .grid_position(chunk_x, chunk_z)
        .seed(seed as i64)
        .builder::<Fbm, Perlin>()
        .seed(seed as i64)
        .octaves(5)
        .frequency(freq * S_FREQ)
        .finalize(true)
        .build();
    res.iter_mut().for_each(|v| *v = (*v * s).abs() + min);
    res
}

pub fn quantize(sample: &mut Vec<f32>, step: f32) {
    sample
        .iter_mut()
        .for_each(|v| *v = (*v / step).round() * step);
}

pub fn mul(a: &mut Vec<f32>, b: &Vec<f32>) {
    a.iter_mut().zip(b).for_each(|(a, b)| *a *= b);
}

pub fn add(a: &mut Vec<f32>, b: &Vec<f32>) {
    a.iter_mut().zip(b).for_each(|(a, b)| *a += b);
}

pub fn add_const(a: &mut Vec<f32>, c: f32) {
    a.iter_mut().for_each(|a| *a += c);
}

pub fn mul_const(a: &mut Vec<f32>, c: f32) {
    a.iter_mut().for_each(|a| *a *= c);
}

pub fn powi(a: &mut Vec<f32>, n: i32) {
    a.iter_mut().for_each(|a| *a = a.powi(n));
}

pub fn points_lerp(a: &mut Vec<f32>, points: &[(f32, f32)]) {
    let max = points.iter().last().unwrap();
    let min = points.iter().next().unwrap();
    let ranges_with_factor = points
        .windows(2)
        .flat_map(|w| {
            if let [(p1, v1), (p2, v2)] = w {
                Some((p1, p2, v1, (v2 - v1) / (p2 - p1)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    a.iter_mut().for_each(|a| {
        if *a < min.0 {
            *a = min.1;
            return;
        }
        if *a >= max.0 {
            *a = max.1;
            return;
        }
        for &(p1, p2, v1, c) in &ranges_with_factor {
            if *p1 <= *a && *a < *p2 {
                *a = v1 + (*a - *p1) * c;
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    const SEEDS: [u32; 8] = [0, 1, 5, 42, 1111111, 3541689516, 1989846551, 62];
    const FREQ: [f32; 5] = [100.0, 10.0, 1.0, 0.1, 0.01];

    /// Asserts that the noise function never returns values outside the given bounds
    /// and that over enough samples the min and max are within 5% of the bounds.
    fn assert_bounds<F>(mut noise: F, min: f32, max: f32)
    where
        F: FnMut(u32, f32) -> Vec<f32>,
    {
        let delta = max - min;
        let mut min_min = f32::INFINITY;
        let mut max_max = f32::NEG_INFINITY;
        for (&seed, &freq) in SEEDS.iter().cartesian_product(FREQ.iter()) {
            let sample = noise(seed, freq);
            let smin = sample.iter().cloned().fold(f32::INFINITY, f32::min);
            let smax = sample.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let bounds = format!("min: {}, max: {}", smin, smax);
            assert!(smin >= min, "{}", bounds);
            assert!(smax <= max, "{}", bounds);
            min_min = min_min.min(smin);
            max_max = max_max.max(smax);
        }
        let bounds = format!("min: {}, max: {}", min_min, max_max);
        assert!((min - min_min).abs() <= 0.05 * delta, "{}", bounds);
        assert!((max - max_max).abs() <= 0.05 * delta, "{}", bounds);
    }

    #[test]
    fn fbm_len() {
        let len = 1024;
        let res = fbm(0, len, 0, len, 42, FREQ[0]);
        assert_eq!(res.len(), len * len);
    }

    #[test]
    fn ridge_len() {
        let len = 1024;
        let res = ridge(0, len, 0, len, 42, FREQ[0]);
        assert_eq!(res.len(), len * len);
    }

    #[test]
    fn fbm_domain() {
        let len = 1024;
        assert_bounds(|seed, freq| fbm(0, len, 0, len, seed, freq), 0., 1.);
    }

    #[test]
    fn ridge_domain() {
        let len = 1024;
        assert_bounds(|seed, freq| ridge(0, len, 0, len, seed, freq), 0., 1.);
    }

    #[test]
    fn fbm_scaled_domain() {
        let len = 1024;
        assert_bounds(
            |seed, freq| fbm_scaled(0, len, 0, len, seed, freq, 50., 100.),
            50.,
            100.,
        );
    }

    #[test]
    fn ridge_scaled_domain() {
        let len = 1024;
        assert_bounds(
            |seed, freq| ridge_scaled(0, len, 0, len, seed, freq, 50., 100.),
            50.,
            100.,
        );
    }
}
