mod log_display;
mod log_inspect;
mod log_replay;

use bevy::prelude::*;
use log_inspect::InspectorPlugin;
use log_replay::LogReplayPlugin;

use crate::log_display::InspectorDisplayPlugin;

fn main() {
    App::new()
        .add_plugins(LogReplayPlugin)
        .add_plugins(InspectorPlugin)
        .add_plugins(InspectorDisplayPlugin)
        .run();
}

#[cfg(test)]
mod tests {
    use bevy::{
        app::{App, Startup},
        ecs::{message::MessageWriter, resource::Resource, system::Res},
    };
    use chrono::{Duration, Utc};
    use rb_logging::{LogData::*, LogEvent};
    use rb_pos::{ChunkPos, ChunkPos2d, Realm};

    use crate::log_inspect::{EventHead, EventQueue, InspectorPlugin, LoadState, MeshCount};

    #[derive(Default, Resource)]
    struct TestEvents(Vec<LogEvent>);

    fn feed_test_events(mut log_events: MessageWriter<LogEvent>, events: Res<TestEvents>) {
        for event in &events.0 {
            log_events.write(event.clone());
        }
    }

    /// Generates a vec of events starting with 1 PlayerMoved event
    /// followed by n-1 random events of type ColGenerated, ChunkMeshed, or ColUnloaded,
    /// all acting on coordinates in [-3, 3].
    fn random_events(n: usize) -> Vec<LogEvent> {
        let mut events = Vec::new();
        events.push(LogEvent {
            timestamp: chrono::Utc::now(),
            data: PlayerMoved {
                id: 42,
                new_col: ChunkPos2d {
                    x: 0,
                    z: 0,
                    realm: Realm::Overworld,
                },
            },
        });
        let now = Utc::now();
        for t in 0..(n - 1) {
            let r = rand::random::<u8>() % 3;
            let x = (rand::random::<i32>() % 7) - 3;
            let z = (rand::random::<i32>() % 7) - 3;
            let data = match r {
                0 => ColGenerated(ChunkPos2d {
                    x,
                    z,
                    realm: Realm::Overworld,
                }),
                1 => ChunkMeshed(ChunkPos {
                    x,
                    y: 0,
                    z,
                    realm: Realm::Overworld,
                }),
                _ => ColUnloaded(ChunkPos2d {
                    x,
                    z,
                    realm: Realm::Overworld,
                }),
            };
            events.push(LogEvent {
                timestamp: now + Duration::milliseconds(t as i64 * 100),
                data,
            });
        }
        events
    }

    fn setup_inspector(events: Vec<LogEvent>) -> App {
        let mut app = App::new();
        app.add_message::<LogEvent>()
            .insert_resource(TestEvents(events))
            .add_systems(Startup, feed_test_events)
            .add_plugins(InspectorPlugin);
        app.update();
        app
    }

    fn move_event_head(app: &mut App, i: usize) {
        let mut event_head = app.world_mut().get_resource_mut::<EventHead>().unwrap();
        event_head.set(i);
        app.update();
    }

    /// Tests that feeding random events then moving the even head randomly then back to 0
    /// results in all columns unloaded and all mesh counts to 0.
    #[test]
    fn test_back_and_forth_to_0() {
        const RANDOM_EVENTS: usize = 400;
        const RANDOM_MOVES: usize = RANDOM_EVENTS * 2;
        let mut app = setup_inspector(random_events(RANDOM_EVENTS));
        let len = app.world().get_resource::<EventQueue>().unwrap().0.len();
        for _ in 0..RANDOM_MOVES {
            let i = rand::random::<u32>() as usize % (len + 1);
            move_event_head(&mut app, i);
        }
        move_event_head(&mut app, 0);
        let load_state = app.world().get_resource::<LoadState>().unwrap();
        // At event head 0, all columns should be unloaded
        assert!(load_state.0.values().all(|v| !v));
        let mesh_count = app.world().get_resource::<MeshCount>().unwrap();
        // At event head 0, all mesh counts should be 0
        assert!(mesh_count.0.values().all(|v| *v == 0));
    }
}
