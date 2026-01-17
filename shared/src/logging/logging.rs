use std::{collections::{HashMap, HashSet}, iter::Rev, ops::{Deref, Range}};
#[cfg(feature = "logging")]
use std::{error::Error, fs::OpenOptions};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use bevy::{log::LogPlugin, prelude::*};
#[cfg(feature = "logging")]
use bevy::log::{tracing, tracing_subscriber::{self, filter::{FromEnvError, ParseError}, fmt, layer::SubscriberExt, EnvFilter, Layer, Registry}};

use crate::world::pos::{pos2d::ColPos, pos3d::ChunkPos};

pub(crate) const LOG_PATH: &'static str = "output.log";

pub struct RiverbedLogPlugin;

impl Plugin for RiverbedLogPlugin {
    fn build(&self, app: &mut App) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "logging")] {
                let log_file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(LOG_PATH)
                    .unwrap();
                let subscriber = Registry::default();
                let filter_layer = EnvFilter::try_from_default_env()
                    .or_else(|from_env_error| {
                        _ = from_env_error
                            .source()
                            .and_then(|source| source.downcast_ref::<ParseError>())
                            .map(|parse_err| {
                                // we cannot use the `error!` macro here because the logger is not ready yet.
                                eprintln!("LogPlugin failed to parse filter from env: {}", parse_err);
                            });

                        Ok::<EnvFilter, FromEnvError>(EnvFilter::builder().parse_lossy("riverbed=trace"))
                    })
                    .unwrap();

                let subscriber = subscriber
                    .with(filter_layer)
                    .with(
                        tracing_subscriber::fmt::layer().
                        with_ansi(false)
                        .with_writer(log_file)
                    );
                if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
                    warn!("Error setting global subscriber: {}", e);
                }
            } else {
                app.add_plugins(LogPlugin::default());
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LogData {
    ColGenerated(ColPos),
    ChunkMeshed(ChunkPos),
    PlayerMoved {
        id: u32, 
        new_col: ColPos
    },
    ColUnloaded(ColPos),
    Message(String)
}

impl std::fmt::Display for LogData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ron::to_string(self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Message, Clone, Debug)]
pub struct LogEvent {
    pub timestamp: DateTime<Utc>,
    pub data: LogData
}

impl std::fmt::Display for LogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ron::to_string(self).unwrap())
    }
}

#[derive(Default, Resource)]
pub struct PlayerPos(pub ColPos);

#[derive(Default, Resource)]
pub struct LoadState(pub HashMap<ColPos, bool>);


#[derive(Resource)]
pub struct IsLive(pub bool);

#[derive(Default, Resource)]
pub struct EventHead {
    previous: usize,
    current: usize
}

impl EventHead {
    pub fn set(&mut self, i: usize) {
        self.previous = self.current;
        self.current = i;
    }

    pub fn moved_forward(&self) -> bool {
        self.current >= self.previous
    }

    pub fn forward_span(&self) -> Range<usize> {
        self.previous..self.current
    }

    pub fn backward_span(&self) -> Rev<Range<usize>> {
        (self.current..self.previous).rev()
    }
}

impl Deref for EventHead {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

#[derive(Default, Resource)]
pub struct EventQueue(pub Vec<LogEvent>);

impl EventQueue {
    pub fn index_at(&self, fraction: f32) -> usize {
        let duration = TimeDelta::milliseconds(
            ((self.0[self.0.len()-1].timestamp - self.0[0].timestamp).num_milliseconds() as f32*fraction) as i64
        );
        let target_timestamp = self.0[0].timestamp+duration;
        match self.0.binary_search_by(|v| v.timestamp.cmp(&target_timestamp)) {
            Ok(i) => i,
            Err(i) => i,
        }
    }
}

#[derive(Default, Resource)]
pub struct MeshCount(pub HashMap<ColPos, u32>);

#[derive(Default, Resource)]
pub struct LiveLoadState(pub HashSet<ColPos>);
