mod blocs;
mod ui;
mod gen;
mod game2d;
mod game3d;
mod agents;
use std::env;
use bevy::{prelude::*, render::texture::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor}};
use blocs::Blocs;
use ui::MenuPlugin;
use game2d::Game2d;
use game3d::Game3d;
use agents::{MovementPlugin, PlayerPlugin};
use gen::GenPlugin;
use ui::UIPlugin;


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Game,
    Menu
}

fn main() {
    let mut app = App::new();

    app
        .init_state::<GameState>()
        .insert_resource(Blocs::new())
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "OurCraft".into(),
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
        .add_plugins(PlayerPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(GenPlugin)
        ;
    
    if env::args().skip(1).any(|arg| arg == "2d") {
        app.add_plugins(Game2d).run();
    } else {
        app.add_plugins(Game3d).run();
    }
}
