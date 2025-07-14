use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;
use bevy::prelude::*;
use chrono::DateTime;
use crate::logging::logging::{LOG_PATH, LogEvent};
use crate::logging::LogData;

pub struct LogReplayPlugin;

impl Plugin for LogReplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LogEvent>()
            .add_systems(Startup, feed_logs)
            ;
    }
}

fn feed_logs(mut log_events: EventWriter<LogEvent>) {
    let Ok(log_lines) = read_lines(LOG_PATH) else {
        panic!("No log file :(");
    };
    for line in log_lines.map_while(Result::ok) {
        let parts: Vec<_> = line.split(' ').filter(|part| part.len() > 0).collect();
        if parts.len() == 0 {
            continue;
        }
        let timestamp = DateTime::from_str(parts[0]).unwrap();
        let data_str = parts[3..].join(" ");
        let data = ron::from_str(&data_str).unwrap_or(LogData::Message(data_str));
        log_events.write(LogEvent {
            timestamp, data
        });
    }
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}