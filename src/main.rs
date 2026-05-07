mod asset_processing;
mod block;
mod items;
mod ui;
mod world;
mod render;
mod agents;
mod sounds;
mod generation;
mod logging;
include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
use avian3d::prelude::*;
use bevy::{image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}, log::LogPlugin, prelude::*, window::PresentMode};
use crossbeam::channel::unbounded;
use world::{GridBlockPos, VoxelWorld};
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use sounds::SoundPlugin;
use ui::UIPlugin;
use render::{BlockTexState, Render, TextureLoadPlugin, TextureMap, spawn_voxel_grid};
use agents::{MovementPlugin, PlayerPlugin};
use world::TerrainLoadPlugin;
#[cfg(feature = "log_inspector")]
use crate::logging::InspectorPlugin;
#[cfg(feature = "log_inspector")]
use crate::logging::LogReplayPlugin;
use crate::{logging::RiverbedLogPlugin, render::{MeshOrderReceiver, MeshOrderSender}};
const SEED: u64 = 42;
pub const RENDER_DISTANCE: i32 = 32;

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
    let (mesh_order_sender, mesh_order_receiver) = unbounded();
    app
        .insert_resource(VoxelWorld::new(mesh_order_sender.clone()))
        .insert_resource(MeshOrderReceiver(mesh_order_receiver))
        .insert_resource(MeshOrderSender(mesh_order_sender))
        .add_plugins(
            DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Riverbed".into(),
                    present_mode: PresentMode::Mailbox,
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
        .insert_resource(WorldRng {
            seed: SEED,
            rng: ChaCha8Rng::seed_from_u64(SEED)
        })
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::new(0., -50., 0.)));
    if std::env::args().any(|a| a == "--debug-physics") {
        app.add_plugins(PhysicsDebugPlugin::default());
    }
    app
        .add_plugins(PlayerPlugin)
        .add_plugins(TextureLoadPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(TerrainLoadPlugin)
        .add_plugins(Render)
        .add_plugins(SoundPlugin)
        .add_systems(OnEnter(BlockTexState::Mapped), spawn_demo_grid)
        ;

    app.run();
}

/// Drops a small cobblestone cube just in front of the player spawn so the
/// voxel-rigidbody pipeline (render, gravity, collisions, break/place) is
/// exercised end-to-end on startup. Strip when no longer needed.
fn spawn_demo_grid(mut commands: Commands, texture_map: Res<TextureMap>) {
    // Player SPAWN is (280, 500, -150) facing +Z. The cube spawns 4m forward
    // and 5m up; 3m³ keeps it visibly comparable to the 1.7m player capsule.
    spawn_voxel_grid(
        &mut commands,
        &texture_map,
        Transform::from_xyz(280., 505., -146.),
        |grid| {
            for x in 0..3 {
                for y in 0..3 {
                    for z in 0..3 {
                        grid.set_block(GridBlockPos { x, y, z }, Block::Cobblestone);
                    }
                }
            }
        },
    );
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