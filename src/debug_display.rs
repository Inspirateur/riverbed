use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use ourcraft::Pos;

use crate::{movement::{Velocity, Heading}, player::Dir};

#[derive(Component)]
struct DebugText;


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "p: \n",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 30.0,
                    color: Color::BEIGE,
                },
            ),
            TextSection::new(
                "v: \n",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 30.0,
                    color: Color::BEIGE,
                },
            ),
            TextSection::new(
                "h: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 30.0,
                    color: Color::BEIGE,
                },
            ),
        ]),
        DebugText,
    ));
}

fn debug_display(mut text_query: Query<&mut Text, With<DebugText>>, player_query: Query<(&Pos, &Velocity, &Heading), With<ActionState<Dir>>>) {
    let (pos, velocity, heading) = player_query.single();
    let mut text = text_query.single_mut();
    text.sections[0].value = format!("p: {:.1}; {:.1}; {:.1}\n", pos.x, pos.y, pos.z);
    text.sections[1].value = format!("v: {:.0}; {:>2.0}; {:.0}\n", velocity.0.x, velocity.0.y, velocity.0.z);
    text.sections[2].value = format!("h: {:.0}; {:.0}; {:.0}", heading.0.x, heading.0.y, heading.0.z);
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, debug_display)
            ;
    }
}