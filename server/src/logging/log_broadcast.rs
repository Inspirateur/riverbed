use bevy::log::trace;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use chrono::Utc;
use crossbeam::channel::{unbounded, Receiver, Sender};
use shared::logging::logging::{LogData, LogEvent};
use shared::messages::ServerToClientMessage;

use crate::network::extensions::SendGameMessageExtension;

/// Channel sender for log events - can be cloned and used from any thread
#[derive(Resource, Clone)]
pub struct LogEventSender(pub Sender<LogEvent>);

/// Channel receiver for log events - used by the broadcast system
#[derive(Resource)]
pub struct LogEventReceiver(pub Receiver<LogEvent>);

pub struct LogBroadcastPlugin;

impl Plugin for LogBroadcastPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = unbounded::<LogEvent>();
        app.insert_resource(LogEventSender(sender));
        app.insert_resource(LogEventReceiver(receiver));
        app.add_systems(Update, broadcast_log_events);
    }
}

/// Extension trait to easily send log events from anywhere
pub trait LogEventSenderExt {
    fn log(&self, data: LogData);
}

impl LogEventSenderExt for LogEventSender {
    fn log(&self, data: LogData) {
        // Also write to tracing (which goes to file when logging feature is enabled)
        trace!("{}", data);

        let event = LogEvent {
            timestamp: Utc::now(),
            data,
        };
        // Ignore send errors - they happen during shutdown
        let _ = self.0.send(event);
    }
}

/// System that broadcasts accumulated log events to all connected clients
fn broadcast_log_events(receiver: Res<LogEventReceiver>, mut server: ResMut<RenetServer>) {
    // Collect all pending events
    let mut events = Vec::new();
    while let Ok(event) = receiver.0.try_recv() {
        events.push(event);
    }

    // Only broadcast if there are events to send
    if !events.is_empty() {
        server.broadcast_game_message(ServerToClientMessage::LogEvents(events));
    }
}
