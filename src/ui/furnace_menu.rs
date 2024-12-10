use bevy::{color::palettes::css, prelude::*};
use crate::agents::Furnace;
use super::{game_menu::despawn_screen, ui_tex_map::{UiSlotKind, UiTextureMap}, FurnaceSlot, GameUiState, ItemHolder, UISlot};

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
    furnace_query: Query<(&Furnace, &ItemHolder)>,
) {
    let Some(furnace_entt) = open_furnace.0 else {
        return;
    };
    let Ok((furnace, ItemHolder::Furnace { fuel, material, output })) = furnace_query.get(furnace_entt) else {
        return;
    };
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            width: Val::Percent(25.),
            height: Val::Percent(80.),
            left: Val::VMin(5.),
            top: Val::VMin(5.),
            ..Default::default()
        },
        BackgroundColor(Color::LinearRgba(LinearRgba::new(0., 0., 0., 0.9))),
    ))
    .with_children(
        |parent| {
            parent.spawn((
                Text::new(&furnace.name),
                TextFont {
                    font_size: 40.,
                    ..Default::default()
                },
                Node {
                    align_self: AlignSelf::Center,
                    ..Default::default()
                }
            ));
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::all(Val::Vw(0.2)),
                ..Default::default()
            }).with_children(|node| {
                node.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Vw(0.2)),
                    ..Default::default()
                }).with_children(|node| {
                    node
                        .spawn(Node {
                            margin: UiRect::all(Val::Vw(0.2)),
                            ..Default::default()
                        })
                        .insert(Interaction::default())
                        .insert(UISlot(furnace_entt, FurnaceSlot::Material.into()))
                        .with_children(|node| tex_map.make_item_slot(node, material, UiSlotKind::Default));
                    node.spawn(Node {
                        margin: UiRect::all(Val::Vw(0.2)),
                        ..Default::default()
                    })
                        .insert(Interaction::default())
                        .insert(UISlot(furnace_entt, FurnaceSlot::Fuel.into()))
                        .with_children(|node| tex_map.make_item_slot(node, fuel, UiSlotKind::Default));
                });
                node.spawn((
                    Text::new("=>"),
                    TextFont{
                        font_size: 40.,
                        ..Default::default()
                    },
                    TextColor(Color::Srgba(css::WHITE)),
                    Node {
                        margin: UiRect::horizontal(Val::Vw(0.1)),
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    }
                ));
                node
                    .spawn(Node {
                        margin: UiRect::all(Val::Vw(0.2)),
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    })
                    .insert(Interaction::default())
                    .insert(UISlot(furnace_entt, FurnaceSlot::Output.into()))
                    .with_children(|node| tex_map.make_item_slot(node, output, UiSlotKind::Default));
            });
    })
    .insert(FurnaceMenu);
}
