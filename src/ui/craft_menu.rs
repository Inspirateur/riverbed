use std::fs;
use bevy::{prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE, color::palettes::css};
use crate::{agents::PlayerControlled, items::{parse_recipes, Hotbar, Ingredient, Inventory, Item, Recipe}};
use super::{game_menu::despawn_screen, hotbar::UiTextureMap, GameUiState};
const SLOT_SIZE_PERCENT: f32 = 4.;

pub struct CraftMenuPlugin;

impl Plugin for CraftMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(HandCrafts(parse_recipes(&fs::read_to_string("assets/data/handcraft.recipe").unwrap())))
            .add_systems(OnEnter(GameUiState::CraftingMenu), open_craft_menu)
            .add_systems(OnExit(GameUiState::CraftingMenu), despawn_screen::<CraftingMenu>)
            ;
    }
}

#[derive(Resource)]
struct HandCrafts(Vec<Recipe>);

#[derive(Component)]
struct CraftingMenu;

fn add_ingredient(parent: &mut ChildBuilder, ingredient: Ingredient, qty: u32, is_craftable: bool, tex_map: &Res<UiTextureMap>) {
    let item = if let Ingredient::Item(item) = ingredient {
        item
    } else {
        Item::Stick
    };
    let alpha = if is_craftable { 1. } else { 0.5 };
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Vw(0.2)),
            ..Default::default()
        },
        ..Default::default()
    }).with_children(|node| {
        node.spawn(ImageBundle {
            style: Style {
                width: Val::Vw(SLOT_SIZE_PERCENT),
                aspect_ratio: Some(1.),
                margin: UiRect::all(Val::Percent(0.2)), 
                ..Default::default()
            },
            image: if let Some(handle) = tex_map.0.get(&item) {
                UiImage::new(handle.clone()).with_color({
                    match item {
                        Item::Block(block) if block.is_foliage() => Color::linear_rgba(0.3, 1.0, 0.1, alpha),
                        _ => Color::linear_rgba(1., 1., 1., alpha)
                    }
                })
            } else {
                UiImage::new(TRANSPARENT_IMAGE_HANDLE)
            },
            background_color: BackgroundColor(if is_craftable {
                Color::linear_rgba(0., 0., 0., 0.3)
            } else {
                Color::NONE
            }),
            ..Default::default()
        });
        node.spawn(TextBundle {
            text: Text::from_section(qty.to_string(), TextStyle { 
                color: if is_craftable {
                    Color::Srgba(css::WHITE)
                } else {
                    Color::Srgba(css::GRAY)
                }, ..Default::default() }),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                ..Default::default()
            },
            ..Default::default() 
        });
    });
}

fn add_recipe_node(parent: &mut ChildBuilder, recipe: Recipe, is_craftable: bool, tex_map: &Res<UiTextureMap>) {
    parent.spawn(NodeBundle {
        style: Style {
            padding: UiRect::all(Val::Percent(0.4)), 
            width: Val::Percent(100.),
            justify_content: JustifyContent::End,
            ..Default::default()
        },
        ..Default::default()
    }).with_children(|node| {
        for (ingredient, qty) in recipe.ingredients {
            add_ingredient(node, ingredient, qty, is_craftable, tex_map);
        }
        node.spawn(TextBundle::from_section("=>", TextStyle { 
            font_size: 40.,
            color: if is_craftable {
                Color::Srgba(css::WHITE)
            } else {
                Color::Srgba(css::GRAY)
            },
            ..Default::default() 
        }));
        add_ingredient(node, Ingredient::Item(recipe.out.0), recipe.out.1, is_craftable, tex_map);
    });
}

fn open_craft_menu(
    mut commands: Commands,
    handcraft_recipes: Res<HandCrafts>,
    hotbar_query: Query<&Hotbar, With<PlayerControlled>>,
    tex_map: Res<UiTextureMap>,
) {
    // TODO: this UI code doesn't produce the exptected result, the images are given no space at all :/
    let empty = Inventory::new();
    let hotbar = hotbar_query.get_single().map(|res| &res.0).unwrap_or(&empty);
    let inventory_recipes = hotbar.filter_recipes(&handcraft_recipes.0);
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(25.),
            height: Val::Percent(80.),
            left: Val::VMin(5.),
            top: Val::VMin(5.),
            ..Default::default()
        },
        background_color: BackgroundColor(Color::LinearRgba(LinearRgba::new(0., 0., 0., 0.4))),
        ..Default::default()
    })
    .with_children(
        |parent| {
            for craftable_recipe in inventory_recipes.craftable_recipes {
                add_recipe_node(parent, craftable_recipe, true, &tex_map);
            }

            for uncraftable_recipe in inventory_recipes.uncraftable_recipes {
                add_recipe_node(parent, uncraftable_recipe, false, &tex_map);
            }
    })
    .insert(CraftingMenu);
}