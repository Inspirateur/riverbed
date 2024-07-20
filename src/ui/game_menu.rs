use bevy::{color::palettes::css, prelude::*};
use crate::GameState;

#[derive(Component)]
struct OnPauseScreen;

pub fn setup_pause(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
    .spawn((
        NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0., 0., 0., 0.6)),
            ..default()
        },
        OnPauseScreen,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_sections([
            TextSection::new(
                "PAUSED",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 100.0,
                    color: Color::Srgba(css::BEIGE),
                }
            )
        ]));
    });
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app 
            .add_systems(OnEnter(GameState::Menu), setup_pause)
            .add_systems(OnExit(GameState::Menu), despawn_screen::<OnPauseScreen>)
            ;
    }
}