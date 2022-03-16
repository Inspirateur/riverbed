use itertools::iproduct;
use noise::NoiseFn;

pub struct Sampler {
    pub data: Vec<f32>,
    noise: Box<dyn NoiseFn<[f64; 2]> + Send + Sync>,
    pub size: u32,
    pub zoom: f32,
}

impl Sampler {
    pub fn new(
        size: u32,
        noise: impl NoiseFn<[f64; 2]> + Send + Sync + 'static,
        zoom: f32,
    ) -> Self {
        let mut sampler = Sampler {
            data: vec![0.; (size * size) as usize],
            noise: Box::new(noise),
            size,
            zoom,
        };
        sampler.do_resample(zoom);
        sampler
    }

    fn do_resample(&mut self, zoom: f32) {
        let sizef = zoom * self.size as f32;
        for (i, (x, y)) in iproduct!(0..self.size, 0..self.size).enumerate() {
            self.data[i] = self
                .noise
                .get([(x as f32 / sizef) as f64, (y as f32 / sizef) as f64])
                as f32;
        }
        self.zoom = zoom;
    }

    pub fn resample(&mut self, zoom: f32) {
        if self.zoom != zoom {
            self.do_resample(zoom);
        }
    }
}
