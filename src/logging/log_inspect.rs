use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use crate::{logging::{logging::LogEvent, LogData}, world::ColPos};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
            .insert_resource(EventQueue::default())
            .insert_resource(IsLive(true))
            .insert_resource(EventHead::default())
            .insert_resource(LastEventHead::default())
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
    mut events: EventReader<LogEvent>, 
    mut event_queue: ResMut<EventQueue>,
    mut event_head: ResMut<EventHead>, 
    mut last_event_head: ResMut<LastEventHead>,
    mut live_load_state: ResMut<LiveLoadState>,
    is_live: Res<IsLive>
) {
    for event in events.read() {
        if let LogData::ColGenerated(col_pos) = event.data {
            if !live_load_state.0.insert(col_pos) {
                println!("Col {:?} generated twice !", col_pos);
                continue;
            }
        } else if let LogData::ColUnloaded(col_pos) = event.data {
            if !live_load_state.0.remove(&col_pos) {
                println!("Col {:?} unloaded twice !", col_pos);
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
    if !is_live.0 || event_queue.0.len() == 0 {
        return;
    }
    last_event_head.0 = event_head.0;
    event_head.0 = event_queue.0.len()-1;
}

fn on_head_change(
    event_queue: Res<EventQueue>,
    event_head: Res<EventHead>,
    last_event_head: Res<LastEventHead>,
    mut player_pos: ResMut<PlayerPos>,
    mut load_state: ResMut<LoadState>,
    mut mesh_count: ResMut<MeshCount>,
) {
    if !event_head.is_changed() {
        return;
    }
    // Find current player pos
    for i in (0..event_head.0).rev() {
        if let LogData::PlayerMoved { id: _, new_col } = event_queue.0[i].data {
            player_pos.0 = new_col;
            break;
        }
    }
    if event_head.0 >= last_event_head.0 {
        // apply the difference between previous index and now
        for i in last_event_head.0..event_head.0 {
            match event_queue.0[i].data {
                LogData::ColGenerated(col) => *load_state.0.entry(col).or_insert(false) = true,
                LogData::ColUnloaded(col) => *load_state.0.get_mut(&col).unwrap() = false,
                LogData::ChunkMeshed(chunk) => *mesh_count.0.entry(chunk.into()).or_insert(0) += 1,
                _ => ()
            }
        }
    } else {
        // We went back in time, apply everything in reverse
        for i in (event_head.0..last_event_head.0).rev() {
            match event_queue.0[i].data {
                LogData::ColGenerated(col) => *load_state.0.get_mut(&col).unwrap() = false,
                LogData::ColUnloaded(col) => *load_state.0.get_mut(&col).unwrap() = true,
                LogData::ChunkMeshed(chunk) => *mesh_count.0.get_mut(&chunk.into()).unwrap() -= 1,
                _ => ()
            }
        }
    }
}

#[derive(Resource)]
struct IsLive(pub bool);

#[derive(Default, Resource)]
struct LastEventHead(usize);

#[derive(Default, Resource)]
struct EventHead(usize);

#[derive(Default, Resource)]
struct EventQueue(pub Vec<LogEvent>);

#[derive(Default, Resource)]
struct PlayerPos(pub ColPos);

#[derive(Default, Resource)]
struct LoadState(pub HashMap<ColPos, bool>);

#[derive(Default, Resource)]
struct MeshCount(pub HashMap<ColPos, u32>);

#[derive(Default, Resource)]
struct LiveLoadState(pub HashSet<ColPos>);