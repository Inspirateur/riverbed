use std::collections::{HashMap, HashSet};
use std::iter::Rev;
use std::ops::{Deref, Range};
use bevy::prelude::*;
use chrono::TimeDelta;
use crate::{logging::{log_display::InspectorDisplayPlugin, logging::LogEvent, LogData}, world::ColPos};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
            .add_plugins(InspectorDisplayPlugin)
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
    let mut recieved_event = false;
    for event in events.read() {
        recieved_event = true;
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
    if !is_live.0 || event_queue.0.len() == 0 || !recieved_event {
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
                LogData::ColUnloaded(col) => *load_state.0.get_mut(&col).unwrap() = false,
                LogData::ChunkMeshed(chunk) => *mesh_count.0.entry(chunk.into()).or_insert(0) += 1,
                _ => ()
            }
        }
    } else {
        // We went back in time, apply everything in reverse
        for i in event_head.backward_span() {
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
pub struct PlayerPos(pub ColPos);

#[derive(Default, Resource)]
pub struct LoadState(pub HashMap<ColPos, bool>);

#[derive(Default, Resource)]
pub struct MeshCount(pub HashMap<ColPos, u32>);

#[derive(Default, Resource)]
struct LiveLoadState(pub HashSet<ColPos>);