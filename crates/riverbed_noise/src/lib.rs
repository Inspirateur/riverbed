use simdnoise::*;
// manual scaling required because library oversight 
// https://github.com/verpeteren/rust-simd-noise/issues/23
const S_FBM: f32 = 3.28;
const S_RIDGE: f32 = 6.58;
const C_RIDGE: f32 = -2.8482;

/// 2D FBM with 3 octaves in [0;1]
pub fn fbm(x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32) -> Vec<f32> {
    let (res, _, _) = NoiseBuilder::fbm_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate();
    res.into_iter().map(|v| v * S_FBM + 0.5).collect()
}

/// 2D Ridge noise in [0;1]
pub fn ridge(x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32) -> Vec<f32> {
    let (res, _, _) = NoiseBuilder::ridge_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate();
    res.into_iter().map(|v| (v + C_RIDGE) * S_RIDGE).collect()
}

/// 2D FBM with 3 octaves in [min;max]
pub fn fbm_scaled(
    x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32, min: f32, max: f32
) -> Vec<f32> {
    let (res, _, _) = NoiseBuilder::fbm_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate();
    res.into_iter().map(|v| (v * S_FBM + 0.5) * (max - min) + min).collect()
}

/// 2D Ridge noise in [min;max]
pub fn ridge_scaled(
    x: f32, width: usize, z: f32, height: usize, seed: u32, freq: f32, min: f32, max: f32
) -> Vec<f32> {
    let (res, _, _) = NoiseBuilder::ridge_2d_offset(x, width, z, height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate();
    res.into_iter().map(|v| (v + C_RIDGE) * S_RIDGE * (max - min) + min).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fbm_domain() {
        let len = 4096;
        let res = fbm(0.0, len, 0.0, len, 42, 0.111111111111);
        assert_eq!(res.len(), len*len);
        let min = res.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = res.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        println!("FBM min: {}, max: {}", min, max);
        assert!(min >= 0.0);
        assert!(min < 0.1);
        assert!(max <= 1.0);
        assert!(max > 0.9);
    }

    #[test]
    fn ridge_domain() {
        let len = 4096;
        let res = ridge(0.0, len, 0.0, len, 42, 0.111111111111);
        assert_eq!(res.len(), len*len);
        let min = res.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = res.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        println!("Ridge min: {}, max: {}", min, max);
        assert!(min >= 0.0);
        assert!(min < 0.1);
        assert!(max <= 1.0);
        assert!(max > 0.9);
    }
}