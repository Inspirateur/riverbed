mod blocs;
mod ui;
mod gen;
mod draw2d;
mod draw3d;
mod sky;
mod agents;
use bevy::{prelude::*, window::{PresentMode, WindowTheme}};
use blocs::Blocs;
use ui::MenuPlugin;
use draw2d::Draw2d;
use draw3d::Draw3d;
use agents::{MovementPlugin, PlayerPlugin};
use gen::LoadTerrainPlugin;
use ui::UIPlugin;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Game,
    Menu
}

fn main() {
    App::new()
        .insert_resource(Blocs::new())
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OurCraft".into(),
                resolution: (1280., 720.).into(),
                present_mode: PresentMode::Fifo,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(UIPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(LoadTerrainPlugin)
        .add_plugins(Draw3d)
        .run();
}
