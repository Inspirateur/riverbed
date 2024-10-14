use bevy::{color::palettes::css, prelude::*};
use crate::agents::Furnace;
use super::{game_menu::despawn_screen, ui_tex_map::UiTextureMap, GameUiState};

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
            align_items: AlignItems::Center,
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
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::all(Val::Vw(0.2)),
                    ..Default::default()
                },
                ..Default::default()
            }).with_children(|node| {
                node.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::all(Val::Vw(0.2)),
                        ..Default::default()
                    },
                    ..Default::default()
                }).with_children(|node| {
                    node.spawn(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Vw(0.2)),
                            ..Default::default()
                        },
                        ..Default::default()
                    }).with_children(|node| tex_map.make_ui_node(node, &furnace.material, false));
                    node.spawn(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Vw(0.2)),
                            ..Default::default()
                        },
                        ..Default::default()
                    }).with_children(|node| tex_map.make_ui_node(node, &furnace.fuel, false));
                });
                node.spawn(TextBundle {
                    text: Text::from_section("=>", TextStyle { 
                        font_size: 40.,
                        color: Color::Srgba(css::WHITE),
                        ..Default::default() 
                    }),
                    style: Style {
                        margin: UiRect::horizontal(Val::Vw(0.1)),
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                });
                node.spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Vw(0.2)),
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                }).with_children(|node| tex_map.make_ui_node(node, &furnace.output, false));
            });
    })
    .insert(FurnaceMenu);
}