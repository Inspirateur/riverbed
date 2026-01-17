use bevy::prelude::*;
use crate::logging::logging::{EventHead, EventQueue, IsLive, LiveLoadState, LoadState, MeshCount, PlayerPos};
use crate::logging::logging::{LogEvent, LogData};

/// Plugin that processes incoming LogEvent messages and maintains inspector state.
/// This can be used by both client (receiving events from server) and for 
/// offline log replay analysis.
pub struct LogInspectorPlugin;

impl Plugin for LogInspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(EventQueue::default())
            .insert_resource(IsLive(true))
            .insert_resource(EventHead::default())
            .insert_resource(PlayerPos::default())
            .insert_resource(LiveLoadState::default())
            .insert_resource(LoadState::default())
            .insert_resource(MeshCount::default())
            .add_systems(Update, on_log_event)
            .add_systems(Update, on_head_change)
            ;
    }
}

fn on_log_event(
    mut events: MessageReader<LogEvent>, 
    mut event_queue: ResMut<EventQueue>,
    mut event_head: ResMut<EventHead>, 
    mut live_load_state: ResMut<LiveLoadState>,
    is_live: Res<IsLive>
) {
    let mut received_event = false;
    for event in events.read() {
        received_event = true;
        if let LogData::ColGenerated(col_pos) = event.data {
            if !live_load_state.0.insert(col_pos) {
                debug!("Col {:?} generated twice!", col_pos);
                continue;
            }
        } else if let LogData::ColUnloaded(col_pos) = event.data {
            if !live_load_state.0.remove(&col_pos) {
                debug!("Col {:?} unloaded twice!", col_pos);
                continue;
            }
        }
        if event_queue.0.len() > 0 && event.timestamp < event_queue.0[event_queue.0.len()-1].timestamp {
            // This event is out of order (probably because of multithreaded tracing), insert it where it should go
            let mut i = event_queue.0.len()-1;
            while i > 0 && event.timestamp < event_queue.0[i-1].timestamp {
                i -= 1;
            }
            if i > 0 {
                i -= 1;
            }
            event_queue.0.insert(i, event.clone());
        } else {
            event_queue.0.push(event.clone());
        }
    }
    if !is_live.0 || event_queue.0.len() == 0 || !received_event {
        return;
    }
    event_head.set(event_queue.0.len()-1);
}

fn on_head_change(
    event_queue: Res<EventQueue>,
    event_head: Res<EventHead>,
    mut player_pos: ResMut<PlayerPos>,
    mut load_state: ResMut<LoadState>,
    mut mesh_count: ResMut<MeshCount>,
) {
    if !event_head.is_changed() {
        return;
    }
    // Find current player pos
    for i in (0..**event_head).rev() {
        if let LogData::PlayerMoved { id: _, new_col } = event_queue.0[i].data {
            player_pos.0 = new_col;
            break;
        }
    }
    if event_head.moved_forward() {
        // apply the difference between previous index and now
        for i in event_head.forward_span() {
            match event_queue.0[i].data {
                LogData::ColGenerated(col) => *load_state.0.entry(col).or_insert(false) = true,
                LogData::ColUnloaded(col) => {
                    if let Some(state) = load_state.0.get_mut(&col) {
                        *state = false;
                    }
                },
                LogData::ChunkMeshed(chunk) => *mesh_count.0.entry(chunk.into()).or_insert(0) += 1,
                _ => ()
            }
        }
    } else {
        // We went back in time, apply everything in reverse
        for i in event_head.backward_span() {
            match event_queue.0[i].data {
                LogData::ColGenerated(col) => {
                    if let Some(state) = load_state.0.get_mut(&col) {
                        *state = false;
                    }
                },
                LogData::ColUnloaded(col) => {
                    if let Some(state) = load_state.0.get_mut(&col) {
                        *state = true;
                    }
                },
                LogData::ChunkMeshed(chunk) => {
                    if let Some(count) = mesh_count.0.get_mut(&chunk.into()) {
                        *count = count.saturating_sub(1);
                    }
                },
                _ => ()
            }
        }
    }
}
