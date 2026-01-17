use bevy::{platform::collections::HashSet, prelude::*};
use shared::messages::ClientPlayerInput;
use shared::net::clock::TickClock;

#[derive(Debug, Default, Resource)]
pub struct PlayerTickInputsBuffer {
    pub buffer: Vec<ClientPlayerInput>,
}

#[derive(Resource, Default)]
pub struct CurrentFrameInputs(pub ClientPlayerInput);

pub trait CurrentFrameInputsExt {
    fn reset(&mut self, time: u64, delta: u64);
}

impl CurrentFrameInputsExt for CurrentFrameInputs {
    fn reset(&mut self, new_time: u64, new_delta: u64) {
        self.0 = ClientPlayerInput {
            time_ms: new_time,
            delta_ms: new_delta,
            inputs: HashSet::default(),
            camera: Transform::default(),
            position: Vec3::default(),
            velocity: Vec3::default(),
            hotbar_slot: 0,
        };
    }
}

#[derive(Resource)]
pub struct SyncTime {
    pub clock: TickClock,
}

impl Default for SyncTime {
    fn default() -> Self {
        Self {
            clock: TickClock::new(),
        }
    }
}

pub trait SyncTimeExt {
    fn delta(&self) -> u64;
    fn advance(&mut self);
    fn set_offset(&mut self, offset_ms: i64);
    fn now_synced(&self) -> i64;
}

impl SyncTimeExt for SyncTime {
    fn delta(&self) -> u64 {
        self.clock.delta()
    }

    fn advance(&mut self) {
        self.clock.advance();
    }

    fn set_offset(&mut self, offset_ms: i64) {
        self.clock.set_offset(offset_ms);
    }

    fn now_synced(&self) -> i64 {
        self.clock.synced_now_ms()
    }
}
