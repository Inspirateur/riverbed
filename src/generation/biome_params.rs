use crate::world::CHUNK_S1;


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