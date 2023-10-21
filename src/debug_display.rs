use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use ourcraft::{Pos, Blocs, Bloc};
use crate::player::{Dir, TargetBloc};

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
                    font_size: 20.0,
                    color: Color::BEIGE,
                },
            ),
            TextSection::new(
                "bloc: \n",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    font_size: 20.0,
                    color: Color::BEIGE,
                },
            ),
        ]),
        DebugText,
    ));
}

fn debug_display(
    mut text_query: Query<&mut Text, With<DebugText>>, 
    player_query: Query<(&Pos, &TargetBloc), With<ActionState<Dir>>>,
    world: Res<Blocs>,
) {
    let (pos, target_bloc) = player_query.single();
    let mut text = text_query.single_mut();
    text.sections[0].value = format!("p: {:.1}; {:.1}; {:.1}\n", pos.x, pos.y, pos.z);
    let bloc = if let Some(raycast_hit) = &target_bloc.0 {
        world.get_block_safe(raycast_hit.pos)
    } else {
        Bloc::Air
    };
    text.sections[1].value = format!("bloc: {bloc:?}");
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, debug_display)
            ;
    }
}