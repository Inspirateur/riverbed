mod terrain_thread;

use bevy::{image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}, log::LogPlugin, prelude::*, window::PresentMode};
use crossbeam::channel::unbounded;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use rb_world::{BlockEntities, ColUnloadEvent, VoxelWorld, WorldRng, unload_block_entities};
use rb_logging::RiverbedLogPlugin;
use rb_sounds::SoundPlugin;
use rb_ui::UIPlugin;
use rb_render::{Render, TextureLoadPlugin, MeshOrderReceiver, MeshOrderSender};
use rb_agents::{MovementPlugin, PlayerPlugin};
use terrain_thread::{setup_load_thread, send_player_pos_update, assign_player_col, on_unload_col};

const SEED: u64 = 42;

pub struct TerrainLoadPlugin;

impl Plugin for TerrainLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ColUnloadEvent>()
            .insert_resource(BlockEntities::default())
            .add_systems(Startup, setup_load_thread)
            .add_systems(Update, send_player_pos_update)
            .add_systems(Update, assign_player_col)
            .add_systems(Update, on_unload_col)
            .add_systems(Update, unload_block_entities);
    }
}

fn main() {
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
        .add_plugins(PlayerPlugin)
        .add_plugins(TextureLoadPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(TerrainLoadPlugin)
        .add_plugins(Render)
        .add_plugins(SoundPlugin)
        ;

    app.run();
}