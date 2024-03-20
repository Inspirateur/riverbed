use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use crate::blocks::{Blocks, Block};
use crate::agents::{Dir, TargetBlock};

fn setup_crosshair(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crosshair = asset_server.load("crosshair.png");
    commands.spawn((
        NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn(ImageBundle {
            style: Style {
                width: Val::Px(34.0),
                ..default()
            },
            image: UiImage::new(crosshair),
            ..default()
        });
    });
}


#[derive(Component)]
struct DebugText;

fn setup_debug_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "p: \n",
                TextStyle {
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 20.0,
                    color: Color::BEIGE,
                },
            ),
            TextSection::new(
                "block: \n",
                TextStyle {
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 20.0,
                    color: Color::BEIGE,
                },
            ),
            TextSection::new(
                "E: \n",
                TextStyle {
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 20.0,
                    color: Color::BEIGE,
                },
            )
        ]).with_style(Style {
            position_type: PositionType::Absolute,
            ..Default::default()
        }),
        DebugText,
    ));
}

fn debug_display(
    mut text_query: Query<&mut Text, With<DebugText>>, 
    player_query: Query<(&Transform, &TargetBlock), With<ActionState<Dir>>>,
    ent_query: Query<Entity, With<Transform>>,
    world: Res<Blocks>,
) {
    let (transform, target_block) = player_query.single();
    let mut text = text_query.single_mut();
    text.sections[0].value = format!("p: {:.1}; {:.1}; {:.1}\n", transform.translation.x, transform.translation.y, transform.translation.z);
    let block = if let Some(raycast_hit) = &target_block.0 {
        world.get_block_safe(raycast_hit.pos)
    } else {
        Block::Air
    };
    text.sections[1].value = format!("block: {block:?}\n");
    let ent_count = ent_query.iter().count();
    text.sections[2].value = format!("E: {ent_count}\n");
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup_crosshair)
        .add_systems(Startup, setup_debug_display)
        .add_systems(Update, debug_display);
    }
}