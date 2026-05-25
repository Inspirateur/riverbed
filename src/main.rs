use bevy::{
    image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor},
    log::LogPlugin,
    prelude::*,
    window::PresentMode,
};
use crossbeam::channel::unbounded;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};
use rb_agents::{PlayerPlugin, TerrainLoadPlugin};
use rb_camera::Camera3dPlugin;
use rb_logging::RiverbedLogPlugin;
use rb_physics::MovementPlugin;
use rb_render::{MeshOrderReceiver, MeshOrderSender, Render, TextureLoadPlugin};
use rb_sounds::SoundPlugin;
use rb_ui::UIPlugin;
use rb_world::{VoxelWorld, WorldRng};

const SEED: u64 = 42;

fn main() {
    let mut app = App::new();
    let (mesh_order_sender, mesh_order_receiver) = unbounded();
    app.insert_resource(VoxelWorld::new(mesh_order_sender.clone()))
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
                })
                .disable::<LogPlugin>(),
        )
        .add_plugins(RiverbedLogPlugin)
        .insert_resource(WorldRng {
            seed: SEED,
            rng: ChaCha8Rng::seed_from_u64(SEED),
        })
        .add_plugins(PlayerPlugin)
        .add_plugins(Camera3dPlugin)
        .add_plugins(TextureLoadPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(TerrainLoadPlugin)
        .add_plugins(Render)
        .add_plugins(SoundPlugin);

    app.run();
}
