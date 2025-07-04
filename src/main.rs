mod asset_processing;
mod block;
mod items;
mod ui;
mod world;
mod render;
mod agents;
mod sounds;
mod terrain;
mod logging;
include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
use bevy::{image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}, prelude::*};
#[cfg(feature = "verbose")]
use bevy::log::Level;
use world::VoxelWorld;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use sounds::SoundPlugin;
use ui::UIPlugin;
use render::{Render, TextureLoadPlugin};
use agents::{MovementPlugin, PlayerPlugin};
use world::GenPlugin;
#[cfg(feature = "log_inspector")]
use crate::logging::InspectorPlugin;
#[cfg(feature = "log_inspector")]
use crate::logging::LogReplayPlugin;
const SEED: u64 = 42;

#[derive(Resource)]
pub struct WorldRng {
    pub seed: u64,
    pub rng: ChaCha8Rng
}

fn main() {
    // TODO: Ideally we would do another executable instead of putting log_inspector in main
    // but this require making a riverbed lib to share structs and I don't want to bother for now
    // see https://doc.rust-lang.org/cargo/reference/features.html#mutually-exclusive-features
    cfg_if::cfg_if! {
        if #[cfg(feature = "log_inspector")] {
            inspect_log();
        } else {
            client();
        }
    }
}

fn client() {
    let mut app = App::new();

    app
        .insert_resource(VoxelWorld::new())
        .add_plugins(
            DefaultPlugins
            .set(WindowPlugin {
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
            .disable::<bevy::log::LogPlugin>()
        )
        .add_plugins(logging::LogPlugin {
            #[cfg(feature = "verbose")]
            level: Level::TRACE,
            #[cfg(feature = "verbose")]
            filter: "warn,riverbed=trace".to_string(),
            ..Default::default()
        })
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
        ;

    app.run();
}

#[cfg(feature = "log_inspector")]
fn inspect_log() {
    let mut app = App::new();

    app
        .add_plugins(LogReplayPlugin)
        .add_plugins(InspectorPlugin)
        .run()
        ;
}