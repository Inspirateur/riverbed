use bevy::prelude::Resource;
use rand_chacha::ChaCha8Rng;

#[derive(Resource)]
pub struct WorldRng {
    pub seed: u64,
    pub rng: ChaCha8Rng
}
