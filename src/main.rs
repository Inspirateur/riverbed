mod col_commands;
mod render2d;
mod draw2d;
mod texture_array;
mod render3d;
mod draw3d;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_cols;
mod player;
mod terrain_gen;
use std::collections::VecDeque;
use bevy::{prelude::*, window::{PresentMode, WindowTheme}};
use ourcraft::{Blocs, Cols};
use col_commands::ColCommands;
use draw2d::Draw2d;
use draw3d::Draw3d;
use leafwing_input_manager::plugin::InputManagerPlugin;
use load_cols::{ColUnloadEvent, ColLoadOrders};
use terrain_gen::Generators;
struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColCommands::new())
            .insert_resource(Blocs(Cols::new()))
            .insert_resource(Generators::new(0))
            .insert_resource(ColLoadOrders(VecDeque::new()))
            .add_event::<ColUnloadEvent>()
            .add_systems(Startup, player::spawn_player)
            .add_systems(Update, player::move_player)
            .add_systems(Update, load_area::update_load_area)
            .add_systems(Update, load_area::load_order)
            .add_systems(Update, load_cols::pull_orders);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OurCraft".into(),
                resolution: (1280., 720.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(InputManagerPlugin::<player::Dir>::default())
        .add_plugins(GameLogic)
        .add_plugins(Draw3d)
        .run();
}
