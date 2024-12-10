mod asset_processing;
mod block;
mod items;
mod ui;
mod world;
mod render;
mod agents;
mod sounds;
mod gen;
include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
use bevy::{image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}, prelude::*};
use world::VoxelWorld;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use sounds::SoundPlugin;
use ui::UIPlugin;
use render::{Render, TextureLoadPlugin};
use agents::{MovementPlugin, PlayerPlugin};
use world::GenPlugin;
const SEED: u64 = 42;

#[derive(Resource)]
pub struct WorldRng {
    pub seed: u64,
    pub rng: ChaCha8Rng
}

fn main() {
    let mut app = App::new();

    app
        .insert_resource(VoxelWorld::new())
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Riverbed".into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Nearest,
                    min_filter: ImageFilterMode::Nearest,
                    mipmap_filter: ImageFilterMode::Nearest,
                    ..default()
                },
            })
        )
        .insert_resource(WorldRng {
            seed: SEED,
            rng: ChaCha8Rng::seed_from_u64(SEED)
        })
        .add_plugins(PlayerPlugin)
        .add_plugins(TextureLoadPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(GenPlugin)
        .add_plugins(Render)
        .add_plugins(SoundPlugin)
        .run()
        ;
}
