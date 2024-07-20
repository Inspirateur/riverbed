use std::fs;
use bevy::{prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE, color::palettes::css};
use crate::{agents::PlayerControlled, items::{parse_recipes, Hotbar, Ingredient, Inventory, Item, Recipe}};
use super::{game_menu::despawn_screen, hotbar::UiTextureMap, GameUiState};
const SLOT_SIZE_PERCENT: f32 = 3.;

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

fn add_ingredient(parent: &mut ChildBuilder, ingredient: Ingredient, qty: u32, tex_map: &Res<UiTextureMap>) {
    let item = if let Ingredient::Item(item) = ingredient {
        item
    } else {
        Item::Stick
    };
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        ..Default::default()
    }).with_children(|node| {
        node.spawn(ImageBundle {
            style: Style {
                width: Val::Percent(SLOT_SIZE_PERCENT),
                aspect_ratio: Some(1.),
                margin: UiRect::all(Val::Percent(0.2)), 
                ..Default::default()
            },
            image: if let Some(handle) = tex_map.0.get(&item) {
                UiImage::new(handle.clone_weak()).with_color({
                    match item {
                        Item::Block(block) if block.is_foliage() => Color::linear_rgba(0.3, 1.0, 0.1, 1.),
                        _ => Color::linear_rgba(1., 1., 1., 1.)
                    }
                })
            } else {
                UiImage::new(TRANSPARENT_IMAGE_HANDLE)
            },
            ..Default::default()
        });
        node.spawn(TextBundle::from_section(qty.to_string(), TextStyle::default()));    
    });
}

fn add_recipe_node(parent: &mut ChildBuilder, recipe: Recipe, is_craftable: bool, tex_map: &Res<UiTextureMap>) {
    parent.spawn(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Percent(0.4)), 
            ..Default::default()
        },
        background_color: BackgroundColor(Color::Srgba(if is_craftable { css::WHITE } else { css::RED })),
        ..Default::default()
    }).with_children(|node| {
        for (ingredient, qty) in recipe.ingredients {
            add_ingredient(node, ingredient, qty, tex_map);
        }
        add_ingredient(node, Ingredient::Item(recipe.out.0), recipe.out.1, tex_map);
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
            ..Default::default()
        },
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