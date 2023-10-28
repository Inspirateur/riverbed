mod sky;
mod render2d;
mod draw2d;
mod texture_array;
mod render3d;
mod draw3d;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_cols;
mod movement;
mod player;
mod terrain_gen;
mod debug_display;
mod menu;
mod ui;
use bevy::{prelude::*, window::{PresentMode, WindowTheme}};
use debug_display::DebugPlugin;
use menu::MenuPlugin;
use ourcraft::Blocs;
use draw2d::Draw2d;
use draw3d::Draw3d;
use leafwing_input_manager::plugin::InputManagerPlugin;
use load_cols::{ColUnloadEvent, LoadedCols};
use terrain_gen::Generators;
use ui::UIPlugin;
struct GameLogic;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Game,
    Menu
}

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadedCols::new())
            .insert_resource(Blocs::new())
            .insert_resource(Generators::new(0))
            .add_state::<GameState>()
            .add_event::<ColUnloadEvent>()
            .add_systems(Startup, player::spawn_player)
            .add_systems(Update, player::move_player.run_if(in_state(GameState::Game)))
            .add_systems(Update, menu::cursor_grab)
            .add_systems(Update, movement::apply_acc)
            .add_systems(Update, movement::apply_gravity)
            .add_systems(Update, movement::apply_speed)
            .add_systems(Update, movement::process_jumps)
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
        .add_plugins(InputManagerPlugin::<player::Dir>::default())
        .add_plugins(InputManagerPlugin::<player::UIAction>::default())
        .add_plugins(GameLogic)
        .add_plugins(UIPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(Draw3d)
        .add_plugins(DebugPlugin)
        .run();
}
