use bevy::platform::collections::HashSet;
use bevy::prelude::Resource;

use crate::messages::ClientPlayerInput;

/// Tracks pending and unacknowledged input frames for a client.
#[derive(Debug, Default, Clone, Resource)]
pub struct InputHistory {
    pending: Vec<ClientPlayerInput>,
    pub unacked: Vec<ClientPlayerInput>,
}

impl InputHistory {
    /// Add a frame to the pending queue.
    pub fn push_frame(&mut self, input: ClientPlayerInput) {
        self.pending.push(input);
    }

    /// Move pending frames into a vector for transmission and mark them as unacked.
    pub fn take_pending(&mut self) -> Vec<ClientPlayerInput> {
        let frames = std::mem::take(&mut self.pending);
        self.unacked.extend(frames.clone());
        frames
    }

    /// Remove all inputs with timestamp <= ack_time. Returns count removed.
    pub fn ack_until(&mut self, ack_time: u64) -> usize {
        let before = self.unacked.len();
        self.unacked.retain(|i| i.time_ms > ack_time);
        before - self.unacked.len()
    }

    /// Clear all pending and unacked inputs (e.g., on disconnect).
    pub fn clear_all(&mut self) {
        self.pending.clear();
        self.unacked.clear();
    }
}

/// Deduplicate and order inputs, dropping any at or before `last_processed`.
/// Returns a new Vec sorted by time_ms with unique timestamps.
pub fn dedup_after(inputs: &[ClientPlayerInput], last_processed: u64) -> Vec<ClientPlayerInput> {
    let mut seen = HashSet::new();
    let mut out: Vec<ClientPlayerInput> = inputs
        .iter()
        .filter(|i| i.time_ms > last_processed)
        .filter(|i| seen.insert(i.time_ms))
        .cloned()
        .collect();

    out.sort_by_key(|i| i.time_ms);
    out
}
