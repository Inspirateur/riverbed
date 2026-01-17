use bevy::prelude::Resource;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Resource)]
pub struct WorldRng {
    pub seed: u64,
    pub rng: ChaCha8Rng,
}

impl WorldRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}
