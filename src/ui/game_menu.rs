use bevy::{color::palettes::css, prelude::*};

use super::GameUiState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app 
            .add_systems(OnEnter(GameUiState::InGameMenu), setup_pause)
            .add_systems(OnExit(GameUiState::InGameMenu), despawn_screen::<OnPauseScreen>)
            ;
    }
}

#[derive(Component)]
struct OnPauseScreen;

pub fn setup_pause(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
    .spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0., 0., 0., 0.6)),
        OnPauseScreen,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("PAUSED"),
            TextFont {
                font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                font_size: 100.0,
                ..Default::default()
            },
            TextColor(Color::Srgba(css::BEIGE))
        ));
    });
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
