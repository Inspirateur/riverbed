mod ui;
mod render;
mod agents;
mod network;
mod sounds;
mod logging;
mod world;

// Re-export Block and BlockFamily from shared crate
pub use shared::block::{Block, BlockFamily};

use bevy::{asset::AssetPlugin, image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}, log::LogPlugin, prelude::*, window::PresentMode};
use crossbeam::channel::unbounded;
use shared::logging::logging::RiverbedLogPlugin;
use shared::world::block_entities::BlockEntities;
use shared::world::world_rng::WorldRng;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use sounds::SoundPlugin;
use ui::UIPlugin;
use render::{Render, TextureLoadPlugin};
use agents::{MovementPlugin, OtherPlayersPlugin, PlayerPlugin};
use network::NetworkPlugin;
use world::{ClientWorldMap, ClientWorldPlugin};
#[cfg(feature = "logging")]
use crate::logging::{InspectorDisplayPlugin, LogInspectorPlugin};
use crate::{render::{MeshOrderReceiver, MeshOrderSender}};
const SEED: u64 = 42;
pub const RENDER_DISTANCE: i32 = 32;

fn main() {
    // TODO: Ideally we would do another executable instead of putting log_inspector in main
    // but this require making a riverbed lib to share structs and I don't want to bother for now
    // see https://doc.rust-lang.org/cargo/reference/features.html#mutually-exclusive-features
    cfg_if::cfg_if! {
        if #[cfg(feature = "logging")] {
            inspect_log();
        } else {
            client();
        }
    }
}

fn client() {
    let mut app = App::new();
    let (mesh_order_sender, mesh_order_receiver) = unbounded();
    app
        .insert_resource(ClientWorldMap::new(mesh_order_sender.clone()))
        .insert_resource(MeshOrderReceiver(mesh_order_receiver))
        .insert_resource(MeshOrderSender(mesh_order_sender))
        .init_resource::<BlockEntities>()
        .add_plugins(ClientWorldPlugin)
        .add_plugins(
            DefaultPlugins
            .set(AssetPlugin {
                // Assets folder is at repo root, not in client folder
                file_path: "../assets".to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Riverbed".into(),
                    present_mode: PresentMode::AutoVsync,
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
            }).disable::<LogPlugin>()
        )
        .add_plugins(RiverbedLogPlugin)
        .add_plugins(NetworkPlugin)
        .insert_resource(WorldRng {
            seed: SEED,
            rng: ChaCha8Rng::seed_from_u64(SEED)
        })
        .add_plugins(PlayerPlugin)
        .add_plugins(OtherPlayersPlugin)
        .add_plugins(TextureLoadPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(Render)
        .add_plugins(SoundPlugin)
        ;

    app.run();
}

#[cfg(feature = "logging")]
fn inspect_log() {
    use shared::logging::LogReplayPlugin;
    
    let mut app = App::new();

    app
        // LogReplayPlugin reads from log file and writes LogEvent messages
        .add_plugins(LogReplayPlugin)
        // LogInspectorPlugin processes LogEvent messages into inspector state
        .add_plugins(LogInspectorPlugin)
        // InspectorDisplayPlugin displays the inspector UI
        .add_plugins(InspectorDisplayPlugin)
        .run()
        ;
}