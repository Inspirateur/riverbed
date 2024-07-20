mod blocks;
mod items;
mod ui;
mod gen;
mod render;
mod agents;
mod sounds;
use bevy::{prelude::*, render::texture::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}};
use blocks::Blocks;
use sounds::SoundPlugin;
use ui::UIPlugin;
use render::{Render, TextureLoadPlugin};
use agents::{MovementPlugin, PlayerPlugin};
use gen::GenPlugin;


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Game,
    Menu,
}

fn main() {
    let mut app = App::new();

    app
        .insert_resource(Blocks::new())
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
        // Note: all init_state needs to be after DefaultPlugins has been added
        .init_state::<GameState>()
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
