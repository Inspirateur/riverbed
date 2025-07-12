use std::time::Duration;

use bevy::color::palettes::css;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::platform::time;
use bevy::prelude::*;
use crate::Block;
use crate::world::VoxelWorld;
use crate::agents::{PlayerControlled, TargetBlock};

pub struct DebugDisplayPlugin;

impl Plugin for DebugDisplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_debug_display)
            .add_systems(Update, update_fps_display)
            .add_systems(Update, update_entt_display)
            .add_systems(Update, update_pos_display)
            .add_systems(Update, update_block_display)
            ;
    }
}

#[derive(Component)]
struct DebugTextFPS;

#[derive(Component)]
struct DebugTextPos;

#[derive(Component)]
struct DebugTextBlock;

#[derive(Component)]
struct DebugTextEntities;

fn setup_debug_display(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        parent.spawn((Text::new("FPS: "), DebugTextFPS));
        parent.spawn((Text::new("p: "), DebugTextPos));
        parent.spawn((Text::new("block: "), DebugTextBlock));
        parent.spawn((Text::new("E: "), DebugTextEntities));
    });
}

fn update_fps_display(
    mut fps_text_query: Query<&mut Text, With<DebugTextFPS>>, 
    diagnostic: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut time_since_rerender: Local<Duration>,
) {
    *time_since_rerender += time.delta();
    if *time_since_rerender < Duration::from_secs(1) {
        return;
    }
    *time_since_rerender = Duration::ZERO;
    if let Ok(mut fps_text) = fps_text_query.single_mut() {
        fps_text.0 = format!(
            "FPS: {}\n", 
            diagnostic.get(&FrameTimeDiagnosticsPlugin::FPS)
                .map_or(0.0, |fps| fps.smoothed().unwrap_or(0.0))
                .round() as u32
        );
    }
}

fn update_entt_display(
    mut entities_text_query: Query<&mut Text, With<DebugTextEntities>>, 
    ent_query: Query<Entity, With<Transform>>,
) {
    let ent_count = ent_query.iter().count();
    if let Ok(mut entities_text) = entities_text_query.single_mut() {
        entities_text.0 = format!("E: {ent_count}\n");
    }
}

fn update_pos_display(
    mut pos_text_query: Query<&mut Text, With<DebugTextPos>>, 
    player_query: Query<&Transform, With<PlayerControlled>>,
) {
    let transform = player_query.single().unwrap();
    if let Ok(mut pos_text) = pos_text_query.single_mut() {
        pos_text.0 = format!("p: {:.1}; {:.1}; {:.1}\n", transform.translation.x, transform.translation.y, transform.translation.z);
    }
}

fn update_block_display(
    player_query: Query<&TargetBlock, With<PlayerControlled>>,
    mut block_text_query: Query<&mut Text, With<DebugTextBlock>>, 
    world: Res<VoxelWorld>,
) {
    let target_block = player_query.single().unwrap();
    let block = if let Some(raycast_hit) = &target_block.0 {
        world.get_block_safe(raycast_hit.pos)
    } else {
        Block::Air
    };
    if let Ok(mut block_text) = block_text_query.single_mut() {
        block_text.0 = format!("block: {block:?}\n");
    }
}