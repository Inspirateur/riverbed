use bevy::prelude::*;

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

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_crosshair);
    }
}