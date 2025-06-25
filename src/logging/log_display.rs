use std::ops::DerefMut;
use crate::logging::log_inspect::{PlayerPos, LoadState, MeshCount, EventQueue, EventHead, IsLive};
use leafwing_input_manager::prelude::*;
use bevy::prelude::*;
use bevy::color::palettes::css;

pub struct InspectorDisplayPlugin;

impl Plugin for InspectorDisplayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugins(
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Riverbed".into(),
                        ..default()
                    }),
                    ..default()
                })
            )
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_plugins(InputManagerPlugin::<TimeControl>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update_camera)
            .add_systems(Update, time_control)
            .add_systems(Update, zoom_camera)
            .add_systems(Update, display_state)
            ;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        InputMap::default().with_axis(CameraMovement::Zoom, MouseScrollAxis::Y)
    ));
    commands.spawn(InputMap::new([
        (TimeControl::Time0, KeyCode::Numpad0),
        (TimeControl::Time1, KeyCode::Numpad1),
        (TimeControl::Time2, KeyCode::Numpad2),
        (TimeControl::Time3, KeyCode::Numpad3),
        (TimeControl::Time4, KeyCode::Numpad4),
        (TimeControl::Time5, KeyCode::Numpad5),
        (TimeControl::Time6, KeyCode::Numpad6),
        (TimeControl::Time7, KeyCode::Numpad7),
        (TimeControl::Time8, KeyCode::Numpad8),
        (TimeControl::Time9, KeyCode::Numpad9),
        (TimeControl::Forward, KeyCode::ArrowRight),
        (TimeControl::Backward, KeyCode::ArrowLeft),
        (TimeControl::GoLive, KeyCode::Enter)
    ]));
}

fn update_camera(player_pos: Res<PlayerPos>, mut cam_query: Query<&mut Transform, With<Camera2d>>) {
    if !player_pos.is_changed() {
        return;
    }
    let mut transform = cam_query.single_mut().unwrap();
    transform.translation.x = player_pos.0.x as f32;
    transform.translation.y = player_pos.0.z as f32;
}

fn zoom_camera(
    cam_query: Single<(&mut Projection, &ActionState<CameraMovement>), With<Camera2d>>,
) {
    const CAMERA_ZOOM_RATE: f32 = 0.05;
    let (mut camera_projection, action_state) = cam_query.into_inner();
    let zoom_delta = action_state.value(&CameraMovement::Zoom);
    match camera_projection.deref_mut() {
        Projection::Orthographic(orthographic_projection) => {
            orthographic_projection.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
        }
        _ => unreachable!(),
    }
}

fn display_state(
    player_pos: Res<PlayerPos>,
    load_state: Res<LoadState>,
    mesh_count: Res<MeshCount>,
    mut gizmos: Gizmos
) {
    for (col, &loaded) in load_state.0.iter() {
        let col_isometry = Isometry2d::from_translation(Vec2::new(col.x as f32, col.z as f32));
        if loaded {
            if *mesh_count.0.get(&col).unwrap_or(&0) > 0 {
                gizmos.rect_2d(col_isometry, Vec2::new(0.8, 0.8), css::SEA_GREEN);
            } else {
                gizmos.rect_2d(col_isometry, Vec2::new(0.8, 0.8), css::STEEL_BLUE);
            }
        } else {
            gizmos.rect_2d(col_isometry, Vec2::new(0.8, 0.8), css::DIM_GRAY);
        }
    }
    let player_isometry = Isometry2d::from_translation(Vec2::new(player_pos.0.x as f32, player_pos.0.z as f32));
    gizmos.circle_2d(player_isometry, 0.3, css::LIGHT_GRAY);
}

fn time_control(
    event_queue: Res<EventQueue>,
    mut event_head: ResMut<EventHead>,
    mut is_live: ResMut<IsLive>,
    query: Single<&ActionState<TimeControl>>
) {
    if event_queue.0.len() == 0 {
        return;
    }
    let action_state = query.into_inner();
    if action_state.just_pressed(&TimeControl::Time0) {
        event_head.set(0);
    } else if action_state.just_pressed(&TimeControl::Time1) {
        event_head.set(event_queue.index_at(0.11));
    } else if action_state.just_pressed(&TimeControl::Time2) {
        event_head.set(event_queue.index_at(0.22));
    } else if action_state.just_pressed(&TimeControl::Time3) {
        event_head.set(event_queue.index_at(0.33));
    } else if action_state.just_pressed(&TimeControl::Time4) {
        event_head.set(event_queue.index_at(0.44));
    } else if action_state.just_pressed(&TimeControl::Time5) {
        event_head.set(event_queue.index_at(0.55));
    } else if action_state.just_pressed(&TimeControl::Time6) {
        event_head.set(event_queue.index_at(0.66));
    } else if action_state.just_pressed(&TimeControl::Time7) {
        event_head.set(event_queue.index_at(0.77));
    } else if action_state.just_pressed(&TimeControl::Time8) {
        event_head.set(event_queue.index_at(0.88));
    } else if action_state.just_pressed(&TimeControl::Time9) {
        event_head.set(event_queue.index_at(0.99));
    }
    if action_state.pressed(&TimeControl::Forward) {
        let i = **event_head;
        if i + 1 < event_queue.0.len() {
            event_head.set(i + 1);
        }
    } else if action_state.pressed(&TimeControl::Backward) {
        let i = **event_head;
        if i > 0 {
            event_head.set(i - 1);
        }
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
pub enum CameraMovement {
    #[actionlike(Axis)]
    Zoom,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
pub enum TimeControl {
    Time0,
    Time1,
    Time2,
    Time3,
    Time4,
    Time5,
    Time6,
    Time7,
    Time8,
    Time9,
    Forward,
    Backward,
    GoLive
}