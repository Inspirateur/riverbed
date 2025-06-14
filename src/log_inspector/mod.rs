use bevy::prelude::*;
use crate::logging::LogEvent;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    // TODO: make another executable that will feed output.log to the analyser
    fn build(&self, app: &mut bevy::prelude::App) {
		app
            .add_systems(Update, log_printer)
			;
    }
}

fn log_printer(mut events: EventReader<LogEvent>) {
    for event in events.read() {
        println!("log to event: {}", event);
    }
}