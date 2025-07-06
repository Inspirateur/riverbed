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
            .add_systems(Update, display_chunk_state)
            .add_systems(Update, display_info)
            ;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        TextFont {
            font: asset_server.load("fonts/RobotoMono-Light.ttf"),
            font_size: 20.0,
            ..Default::default()
        },
        TextColor(Color::Srgba(css::BEIGE)),
    )).with_children(|parent| {
        parent.spawn((Text::new("time: "), TextDuration));
        parent.spawn((Text::new("pos: "), TextPlayerPos));
        parent.spawn((Text::new("meshing count: "), TextMeshCount));
        parent.spawn((Text::new("mesh throughput: "), TextMeshThroughput));
        parent.spawn((Text::new("avg mesh per col: | max:"), TextColumnMesh));
    });
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

fn display_chunk_state(
    player_pos: Res<PlayerPos>,
    load_state: Res<LoadState>,
    mesh_count: Res<MeshCount>,
    mut gizmos: Gizmos
) {
    for (col, &loaded) in load_state.0.iter() {
        let col_isometry = Isometry2d::from_translation(Vec2::new(col.x as f32, col.z as f32));
        if loaded {
            let count = *mesh_count.0.get(&col).unwrap_or(&0);
            if count > 0 {
                let redness = 1.-(-(count as f32)/8.).exp();
                let hue = 120.-redness*120.;
                gizmos.rect_2d(col_isometry, Vec2::new(0.8, 0.8), Color::hsv(hue, 0.8, 0.8));
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

fn display_info(
    player_pos: Res<PlayerPos>,
    mesh_count: Res<MeshCount>,
    event_queue: Res<EventQueue>,
    event_head: Res<EventHead>,
    mut text_query: ParamSet<(
        Single<&mut Text, With<TextDuration>>,
        Single<&mut Text, With<TextPlayerPos>>,
        Single<&mut Text, With<TextMeshCount>>,
        Single<&mut Text, With<TextMeshThroughput>>,
        Single<&mut Text, With<TextColumnMesh>>,
    )>,
) {
    let duration = event_queue.0[**event_head].timestamp - event_queue.0[0].timestamp;
    let total_meshed = mesh_count.0.values().fold(0, |a, b| a + b);
    let avg_mesh_count = total_meshed as f32/mesh_count.0.values().filter(|&&v| v > 0).count() as f32;
    let max_mesh_count = *mesh_count.0.values().max().unwrap_or(&0);
    let mut text_duration = text_query.p0().into_inner();
    text_duration.0 = format!("time: {:.2} sec", duration.as_seconds_f32());
    let mut text_pos = text_query.p1().into_inner();
    text_pos.0 = format!("column pos: {} ; {}", player_pos.0.x, player_pos.0.z);
    let mut text_mesh_total = text_query.p2().into_inner();
    text_mesh_total.0 = format!("meshing count: {}", total_meshed);
    let mut text_mesh_thoughput = text_query.p3().into_inner();
    text_mesh_thoughput.0 = format!("mesh throughput: {:.0}/s", total_meshed as f32/duration.as_seconds_f32());
    let mut text_max_col_mesh = text_query.p4().into_inner();
    text_max_col_mesh.0 = format!("avg mesh per col: {:.1} | max: {}", avg_mesh_count, max_mesh_count);
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
    let mut time_skipped = true;
    let action_state = query.into_inner();
    if action_state.just_pressed(&TimeControl::Time0) {
        event_head.set(0);
    } else if action_state.just_pressed(&TimeControl::Time1) {
        event_head.set(event_queue.index_at(1./9.));
    } else if action_state.just_pressed(&TimeControl::Time2) {
        event_head.set(event_queue.index_at(2./9.));
    } else if action_state.just_pressed(&TimeControl::Time3) {
        event_head.set(event_queue.index_at(3./9.));
    } else if action_state.just_pressed(&TimeControl::Time4) {
        event_head.set(event_queue.index_at(4./9.));
    } else if action_state.just_pressed(&TimeControl::Time5) {
        event_head.set(event_queue.index_at(5./9.));
    } else if action_state.just_pressed(&TimeControl::Time6) {
        event_head.set(event_queue.index_at(6./9.));
    } else if action_state.just_pressed(&TimeControl::Time7) {
        event_head.set(event_queue.index_at(7./9.));
    } else if action_state.just_pressed(&TimeControl::Time8) {
        event_head.set(event_queue.index_at(8./9.));
    } else if action_state.just_pressed(&TimeControl::Time9) {
        event_head.set(event_queue.0.len()-1);
    } else if action_state.pressed(&TimeControl::Forward) {
        let i = **event_head;
        if i + 1 < event_queue.0.len() {
            event_head.set(i + 1);
        }
    } else if action_state.pressed(&TimeControl::Backward) {
        let i = **event_head;
        if i > 0 {
            event_head.set(i - 1);
        }
    } else {
        time_skipped = false;
    }
    if time_skipped {
        is_live.0 = false;
    }
    if action_state.just_pressed(&TimeControl::GoLive) {
        is_live.0 = true;
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
enum CameraMovement {
    #[actionlike(Axis)]
    Zoom,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect, Hash)]
enum TimeControl {
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

#[derive(Component)]
struct TextPlayerPos;

#[derive(Component)]
struct TextMeshCount;

#[derive(Component)]
struct TextDuration;

#[derive(Component)]
struct TextMeshThroughput;

#[derive(Component)]
struct TextColumnMesh;