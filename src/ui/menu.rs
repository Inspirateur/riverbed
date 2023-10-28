use bevy::{prelude::*, window::CursorGrabMode};
use leafwing_input_manager::prelude::ActionState;
use crate::{agents::UIAction, GameState};

#[derive(Component)]
struct OnPauseScreen;

pub fn cursor_grab(
    mut windows: Query<&mut Window>,
    mut ui_action_query: Query<&ActionState<UIAction>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>
) {
    if let Ok(mut window) = windows.get_single_mut() {
        let action_state = ui_action_query.single_mut();
        for action in action_state.get_just_pressed() {
            if action == UIAction::Escape {
                if **game_state == GameState::Menu {
                    println!("unpaused");
                    window.cursor.visible = false;
                    window.cursor.grab_mode = CursorGrabMode::Confined;
                    next_game_state.set(GameState::Game);
                } else {
                    println!("paused");
                    window.cursor.visible = true;
                    window.cursor.grab_mode = CursorGrabMode::None;
                    next_game_state.set(GameState::Menu);
                }
            }
        }
    }
}

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
            background_color: BackgroundColor(Color::Rgba { red: 0., green: 0., blue: 0., alpha: 0.6 }),
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
                    color: Color::BEIGE,
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
            .add_systems(Update, cursor_grab)
            .add_systems(OnEnter(GameState::Menu), setup_pause)
            .add_systems(OnExit(GameState::Menu), despawn_screen::<OnPauseScreen>)
            ;
    }
}