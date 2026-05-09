mod log_inspect;
mod log_replay;
mod log_display;

use log_inspect::InspectorPlugin;
use log_replay::LogReplayPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(LogReplayPlugin)
        .add_plugins(InspectorPlugin)
        .run();
}