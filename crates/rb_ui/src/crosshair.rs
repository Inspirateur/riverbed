use bevy::prelude::*;

pub fn setup_crosshair(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crosshair = asset_server.load("crosshair.png");
    commands.spawn(Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            ImageNode {
                image: crosshair,
                ..default()
            },
            Node {
                width: Val::Px(34.0),
                ..default()
            },
        ));
    });
}
