use bevy::{platform::collections::HashSet, prelude::*};
use shared::{messages::PlayerFrameInput};

/// Buffer of player inputs collected during the current tick, waiting to be sent to the server.
#[derive(Debug, Default, Resource)]
pub struct PlayerTickInputsBuffer {
    pub buffer: Vec<PlayerFrameInput>,
}

/// The current frame's input state being built up.
/// 
/// Each frame, inputs are captured and accumulated here. At the end of the frame,
/// this is moved to `PlayerTickInputsBuffer` and reset for the next frame.
#[derive(Resource, Default)]
pub struct CurrentFrameInputs(pub PlayerFrameInput);

pub trait CurrentFrameInputsExt {
    fn reset(&mut self, time: u64, delta: u64);
}

impl CurrentFrameInputsExt for CurrentFrameInputs {
    fn reset(&mut self, new_time: u64, new_delta: u64) {
        self.0 = PlayerFrameInput {
            time_ms: new_time,
            delta_ms: new_delta,
            inputs: HashSet::default(),
            camera: Transform::default(),
            position: Vec3::default(),
            hotbar_slot: 0,
        };
    }
}

/// Wall-clock time tracking for input synchronization.
/// 
/// # Time Concepts in Riverbed
/// 
/// There are several time-related values used in the networking system:
/// 
/// - **`SyncTime`** (this): Wall-clock time in milliseconds, used to timestamp player inputs.
///   Currently assumes client and server clocks are roughly synchronized. A proper NTP-like
///   system should be implemented for high-latency scenarios.
/// 
/// - **`ServerTick`** (server): Logical game tick counter incremented each fixed timestep.
///   Used to order game events deterministically.
/// 
/// - **`ServerTickAtConnect`** (client): The server's tick count at the moment of authentication.
///   Can be used to calculate how many ticks have elapsed since connection.
/// 
/// - **`timestamp_ms`** in auth response: Server's wall-clock time at authentication.
///   Reserved for future clock synchronization implementation.
#[derive(Resource)]
pub struct SyncTime {
    pub last_time_ms: u64,
    pub curr_time_ms: u64,
}

impl Default for SyncTime {
    fn default() -> Self {
        let current_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            last_time_ms: current_time_ms,
            curr_time_ms: current_time_ms,
        }
    }
}

pub trait SyncTimeExt {
    fn delta(&self) -> u64;
    fn advance(&mut self);
}

impl SyncTimeExt for SyncTime {
    fn delta(&self) -> u64 {
        self.curr_time_ms - self.last_time_ms
    }

    fn advance(&mut self) {
        self.last_time_ms = self.curr_time_ms;
        self.curr_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }
}
