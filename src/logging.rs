use std::{error::Error, sync::mpsc, fs::OpenOptions};
use serde::{Deserialize, Serialize};
use bevy::{log::{tracing::{self, subscriber, Subscriber}, tracing_subscriber::{self, filter::{FromEnvError, ParseError}, fmt, layer::SubscriberExt, EnvFilter, Layer, Registry}, BoxedLayer, Level, DEFAULT_FILTER}, prelude::*};
use crate::world::{ChunkPos, ColPos};

pub struct LogPlugin {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: Level,
}

impl Default for LogPlugin {
    fn default() -> Self {
        Self {
            filter: DEFAULT_FILTER.to_string(),
            level: Level::INFO,
        }
    }
}

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        let log_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("output.log")
            .unwrap();
        let subscriber = Registry::default();

        // If the inspector is enabled, we add a layer that translates logs to bevy events
        #[cfg(feature = "inspector")]
        let subscriber = subscriber.with(event_transfer_layer(app));

        let default_filter = { format!("{},{}", self.level, self.filter) };
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|from_env_error| {
                _ = from_env_error
                    .source()
                    .and_then(|source| source.downcast_ref::<ParseError>())
                    .map(|parse_err| {
                        // we cannot use the `error!` macro here because the logger is not ready yet.
                        eprintln!("LogPlugin failed to parse filter from env: {}", parse_err);
                    });

                Ok::<EnvFilter, FromEnvError>(EnvFilter::builder().parse_lossy(&default_filter))
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
    }
}

#[derive(Serialize, Deserialize, Event)]
pub enum LogEvent {
    ColGenerated(ColPos),
    ChunkMeshed(ChunkPos),
    PlayerMoved {
        id: u32, 
        new_col: ColPos
    },
    Message(String)
}

impl std::fmt::Display for LogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ron::to_string(self).unwrap())
    }
}

/// This non-send resource temporarily stores [`LogEvent`]s before they are
/// written to [`Events<LogEvent>`] by [`transfer_log_events`].
#[derive(Deref, DerefMut)]
struct CapturedLogEvents(mpsc::Receiver<LogEvent>);


/// Transfers information from the `LogEvents` resource to [`Events<LogEvent>`](LogEvent).
fn transfer_log_events(
    receiver: NonSend<CapturedLogEvents>,
    mut log_events: EventWriter<LogEvent>,
) {
    // Make sure to use `try_iter()` and not `iter()` to prevent blocking.
    log_events.write_batch(receiver.try_iter());
}

/// This is the [`Layer`] that we will use to capture log events and then send them to Bevy's
/// ECS via its [`mpsc::Sender`].
struct CaptureLayer {
    sender: mpsc::Sender<LogEvent>,
}

impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // In order to obtain the log message, we have to create a struct that implements
        // Visit and holds a reference to our string. Then we use the `record` method and
        // the struct to modify the reference to hold the message string.
        let mut message = None;
        event.record(&mut CaptureLayerVisitor(&mut message));
        if let Some(message) = message {
            let _metadata = event.metadata();

            self.sender
                .send(ron::from_str(&message).unwrap_or(LogEvent::Message(message)))
                .expect("LogEvents resource no longer exists!");
        }
    }
}

/// A [`Visit`](tracing::field::Visit)or that records log messages that are transferred to [`CaptureLayer`].
struct CaptureLayerVisitor<'a>(&'a mut Option<String>);

impl tracing::field::Visit for CaptureLayerVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // This if statement filters out unneeded events sometimes show up
        if field.name() == "message" {
            *self.0 = Some(format!("{value:?}"));
        }
    }
}

pub(crate) fn event_transfer_layer(app: &mut App) -> Option<BoxedLayer> {
    let (sender, receiver) = mpsc::channel();

    let layer = CaptureLayer { sender };
    let resource = CapturedLogEvents(receiver);

    app.insert_non_send_resource(resource);
    app.add_event::<LogEvent>();
    app.add_systems(Update, transfer_log_events);

    Some(layer.boxed())
}