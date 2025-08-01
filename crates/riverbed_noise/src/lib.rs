use simdnoise::*;
// manual scaling required because library oversight 
// https://github.com/verpeteren/rust-simd-noise/issues/23
const S_FBM: f32 = 0.765;
const S_RIDGE: f32 = 6.58;
const C_RIDGE: f32 = -2.8482;

/// 2D FBM with 3 octaves in [0;1]
pub fn fbm(x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32) -> Vec<f32> {
    let (res, _, _) = NoiseBuilder::fbm_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .with_octaves(5)
        .generate();
    res.into_iter().map(|v| v*S_FBM + 0.5).collect()
}

/// 2D FBM with 3 octaves in [min;max]
pub fn fbm_scaled(
    x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32, min: f32, max: f32
) -> Vec<f32> {
    let delta = max-min;
    let s = S_FBM*delta;
    let c = 0.5*delta + min;
    let (res, _, _) = NoiseBuilder::fbm_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .with_octaves(5)
        .generate();
    res.into_iter().map(|v| v * s + c).collect()
}

/// 2D Ridge noise in [0;1]
pub fn ridge(x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32) -> Vec<f32> {
    let (res, _, _) = NoiseBuilder::ridge_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate();
    res.into_iter().map(|v| (v + C_RIDGE) * S_RIDGE).collect()
}

/// 2D Ridge noise in [min;max]
pub fn ridge_scaled(
    x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32, min: f32, max: f32
) -> Vec<f32> {
    let delta = max - min;
    let s = S_RIDGE*delta;
    let c = S_RIDGE*delta*C_RIDGE + min;
    let (res, _, _) = NoiseBuilder::ridge_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate();
    res.into_iter().map(|v| v * s + c).collect()
}

pub fn quantize(sample: &mut Vec<f32>, step: f32) {
    sample.iter_mut().for_each(|v| *v = (*v/step).round()*step);   
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bounds(sample: Vec<f32>, min: f32, max: f32) {
        let delta = max - min;
        let error = delta * 0.05;
        let smin = sample.iter().cloned().fold(f32::INFINITY, f32::min);
        let smax = sample.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        println!("min: {}, max: {}", smin, smax);
        assert!(smin >= min);
        assert!(smin < min+error);
        assert!(smax <= max);
        assert!(smax > max-error);
    }

    #[test]
    fn fbm_len() {
        let len = 1024;
        let res = fbm(0.0, len, 0.0, len, 42, 0.01);
        assert_eq!(res.len(), len*len);
    }

    #[test]
    fn ridge_len() {
        let len = 1024;
        let res = ridge(0.0, len, 0.0, len, 42, 0.01);
        assert_eq!(res.len(), len*len);
    }

    #[test]
    fn fbm_domain() {
        let len = 4096;
        let res = fbm(0.0, len, 0.0, len, 11111, 0.333333);
        assert_bounds(res, 0., 1.);
    }

    #[test]
    fn ridge_domain() {
        let len = 4096;
        let res = ridge(0.0, len, 0.0, len, 42, 0.333333);
        assert_bounds(res, 0., 1.);
    }

    #[test]
    fn fbm_scaled_domain() {
        let len = 4096;
        let res = fbm_scaled(0.0, len, 0.0, len, 42, 0.333333, 50., 100.);
        assert_bounds(res, 50., 100.);
    }

    #[test]
    fn ridge_scaled_domain() {
        let len = 4096;
        let res = ridge_scaled(0.0, len, 0.0, len, 42, 0.333333, 50., 100.);
        assert_bounds(res, 50., 100.);
    }
}