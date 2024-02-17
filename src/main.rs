mod blocs;
mod ui;
mod gen;
mod game2d;
mod game3d;
mod agents;
use std::env;
use bevy::{prelude::*, window::{PresentMode, WindowTheme}};
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

    app.insert_resource(Blocs::new())
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OurCraft".into(),
                resolution: (1280., 720.).into(),
                present_mode: PresentMode::Immediate,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(PlayerPlugin)
        // .add_plugins(UIPlugin)
        // .add_plugins(MenuPlugin)
        // .add_plugins(MovementPlugin)
        .add_plugins(GenPlugin);

    if env::args().skip(1).any(|arg| arg == "2d") {
        app.add_plugins(Game2d).run();
    } else {
        app.add_plugins(Game3d).run();
    }
}
