use bevy::prelude::*;
use crate::{agents::{Furnace, PlayerControlled}, items::{FiringTable, Hotbar}};
use super::{game_menu::despawn_screen, hotbar::UiTextureMap, GameUiState};

pub struct FurnaceMenuPlugin;

impl Plugin for FurnaceMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(OpenFurnace(None))
            .add_systems(OnEnter(GameUiState::FurnaceMenu), open_furnace_menu)
            .add_systems(OnExit(GameUiState::FurnaceMenu), despawn_screen::<FurnaceMenu>)
            //.add_systems(Update, ().chain().run_if(in_state(GameUiState::FurnaceMenu)))
            ;
    }
}

#[derive(Resource)]
pub struct OpenFurnace(pub Option<Entity>);

#[derive(Component)]
struct FurnaceMenu;

fn open_furnace_menu(
    mut commands: Commands,
    firing_table: Res<FiringTable>,
    hotbar_query: Query<&Hotbar, With<PlayerControlled>>,
    tex_map: Res<UiTextureMap>,
    open_furnace: Res<OpenFurnace>,
    furnace_query: Query<&Furnace>,
) {
    let Some(furnace_entt) = open_furnace.0 else {
        return;
    };
    let Ok(furnace) = furnace_query.get(furnace_entt) else {
        return;
    };
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(25.),
            height: Val::Percent(80.),
            left: Val::VMin(5.),
            top: Val::VMin(5.),
            ..Default::default()
        },
        background_color: BackgroundColor(Color::LinearRgba(LinearRgba::new(0., 0., 0., 0.9))),
        ..Default::default()
    })
    .with_children(
        |parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(&furnace.name, TextStyle {
                    font_size: 40.,
                    ..Default::default()
                }),
                style: Style {
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                ..Default::default()
            });
            parent.spawn(NodeBundle::default())
    })
    .insert(FurnaceMenu);
}