use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Returns the current system time in milliseconds since UNIX epoch.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// A small clock that tracks the latest tick, previous tick, and a remote offset.
/// Offset is signed so it can represent clock skew (remote - local).
#[derive(Debug, Clone, Copy)]
pub struct TickClock {
    pub last_ms: u64,
    pub curr_ms: u64,
    pub offset_ms: i64,
}

impl Default for TickClock {
    fn default() -> Self {
        Self::new()
    }
}

impl TickClock {
    pub fn new() -> Self {
        let now = now_ms();
        Self {
            last_ms: now,
            curr_ms: now,
            offset_ms: 0,
        }
    }

    pub fn with_offset(offset_ms: i64) -> Self {
        let now = now_ms();
        Self {
            last_ms: now,
            curr_ms: now,
            offset_ms,
        }
    }

    pub fn advance(&mut self) {
        self.last_ms = self.curr_ms;
        self.curr_ms = now_ms();
    }

    /// Milliseconds elapsed between the last two `advance` calls.
    pub fn delta(&self) -> u64 {
        self.curr_ms.saturating_sub(self.last_ms)
    }

    /// Current time adjusted by offset (remote-aligned time).
    pub fn synced_now_ms(&self) -> i64 {
        self.curr_ms as i64 + self.offset_ms
    }

    pub fn set_offset(&mut self, offset_ms: i64) {
        self.offset_ms = offset_ms;
    }
}

/// Compute the offset between a remote timestamp and a local timestamp.
/// Positive result means the remote clock is ahead of the local clock.
pub fn compute_offset(remote_time_ms: u64, local_time_ms: u64) -> i64 {
    remote_time_ms as i64 - local_time_ms as i64
}

/// Convenience helper: current time as a Duration since UNIX epoch.
pub fn now_duration() -> Duration {
    Duration::from_millis(now_ms())
}
