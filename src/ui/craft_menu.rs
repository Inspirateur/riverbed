use std::fs;
use bevy::{prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE, color::palettes::css};
use leafwing_input_manager::action_state::ActionState;
use crate::{agents::{Action, PlayerControlled}, items::{parse_recipes, Hotbar, Ingredient, Inventory, InventoryRecipes, Item, Recipe, Stack}};
use super::{game_menu::despawn_screen, hotbar::UiTextureMap, GameUiState, UIAction};
const SLOT_SIZE_PERCENT: f32 = 4.;

pub struct CraftMenuPlugin;

impl Plugin for CraftMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedRecipe(0))
            .insert_resource(HandCrafts(parse_recipes(&fs::read_to_string("assets/data/handcraft.recipe").unwrap())))
            .add_systems(OnEnter(GameUiState::CraftingMenu), open_craft_menu)
            .add_systems(OnExit(GameUiState::CraftingMenu), despawn_screen::<CraftingMenu>)
            .add_systems(Update, (
                scroll_recipes, 
                display_selected_recipe,
                craft_action,
                refresh_craft_menu, 
            ).chain().run_if(in_state(GameUiState::CraftingMenu)))
            ;
    }
}

#[derive(Resource)]
struct HandCrafts(Vec<Recipe>);

#[derive(Component)]
struct RecipeSlot(pub usize);

#[derive(Resource)]
pub struct SelectedRecipe(pub usize);

#[derive(Component)]
struct CraftingMenu(pub InventoryRecipes);

fn add_ingredient(parent: &mut ChildBuilder, ingredient: &Ingredient, qty: u32, is_craftable: bool, tex_map: &Res<UiTextureMap>) {
    let item = if let Ingredient::Item(item) = ingredient {
        item
    } else {
        &Item::Stick
    };
    let alpha = if is_craftable { 1. } else { 0.6 };
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

fn add_recipe_node(parent: &mut ChildBuilder, recipe: &Recipe, is_craftable: bool, tex_map: &Res<UiTextureMap>, slot: usize) {
    parent.spawn(NodeBundle {
        style: Style {
            padding: UiRect::all(Val::Percent(0.4)), 
            width: Val::Percent(100.),
            justify_content: JustifyContent::End,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(RecipeSlot(slot))
    .with_children(|node| {
        for (ingredient, qty) in &recipe.ingredients {
            add_ingredient(node, ingredient, *qty, is_craftable, tex_map);
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
        add_ingredient(node, &Ingredient::Item(recipe.out.0), recipe.out.1, is_craftable, tex_map);
    });
}

fn create_craft_menu(mut commands: Commands, inventory_recipes: InventoryRecipes, tex_map: Res<UiTextureMap>) {
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
            let mut i = 0;
            for (craftable_recipe, _) in &inventory_recipes.craftable_recipes {
                add_recipe_node(parent, craftable_recipe, true, &tex_map, i);
                i += 1;
            }

            for uncraftable_recipe in &inventory_recipes.uncraftable_recipes {
                add_recipe_node(parent, uncraftable_recipe, false, &tex_map, i);
                i += 1;
            }
    })
    .insert(CraftingMenu(inventory_recipes));
}

fn open_craft_menu(
    commands: Commands,
    handcraft_recipes: Res<HandCrafts>,
    hotbar_query: Query<&Hotbar, With<PlayerControlled>>,
    tex_map: Res<UiTextureMap>,
) {
    println!("open");
    let empty = Inventory::new();
    let hotbar = hotbar_query.get_single().map(|res| &res.0).unwrap_or(&empty);
    let inventory_recipes = hotbar.filter_recipes(&handcraft_recipes.0);
    create_craft_menu(commands, inventory_recipes, tex_map);
}

fn refresh_craft_menu(
    mut commands: Commands,
    handcraft_recipes: Res<HandCrafts>,
    hotbar_query: Query<&Hotbar, (With<PlayerControlled>, Changed<Hotbar>)>,
    tex_map: Res<UiTextureMap>,
    craft_menu_query: Query<Entity, With<CraftingMenu>>,
) {
    let Ok(hotbar) = hotbar_query.get_single() else {
        return;
    };
    if let Ok(entity) = craft_menu_query.get_single() {
        commands.entity(entity).despawn_recursive();
    };
    println!("refresh");
    let inventory_recipes = hotbar.0.filter_recipes(&handcraft_recipes.0);
    create_craft_menu(commands, inventory_recipes, tex_map);
}

fn display_selected_recipe(
    selected_recipe: Res<SelectedRecipe>,
    mut recipe_query: Query<(&RecipeSlot, &mut BackgroundColor)>,
    craft_menu_query: Query<&CraftingMenu>,
) {
    let Ok(craft_menu) = craft_menu_query.get_single() else {
        return;
    };
    let slots = craft_menu.0.craftable_recipes.len();
    for (slot, mut bg) in recipe_query.iter_mut() {
        if selected_recipe.0 < slots && slot.0 == selected_recipe.0 {
            bg.0 = Color::linear_rgba(1., 1., 1., 0.3);
        } else {
            bg.0 = Color::NONE;
        }
    }
}

fn scroll_recipes(
    mut selected_recipe: ResMut<SelectedRecipe>,
    action_query: Query<&ActionState<UIAction>>,
    craft_menu_query: Query<&CraftingMenu>
) {
    let Ok(craft_menu) = craft_menu_query.get_single() else {
        return;
    };
    let slots = craft_menu.0.craftable_recipes.len();
    if slots == 0 {
        return;
    }
    let Ok(action_state) = action_query.get_single() else {
        return;
    };
    if action_state.pressed(&UIAction::ScrollUp) {
        selected_recipe.0 = (selected_recipe.0 - 1).rem_euclid(slots);
    } else if action_state.pressed(&UIAction::ScrollDown) {
        selected_recipe.0 = (selected_recipe.0 + 1).rem_euclid(slots);
    }
}

fn craft_action(
    selected_recipe: Res<SelectedRecipe>,
    craft_menu_query: Query<&CraftingMenu>,
    mut hotbar_query: Query<&mut Hotbar, With<PlayerControlled>>,
    action_query: Query<&ActionState<Action>>,
) {
    let Ok(action_state) = action_query.get_single() else {
        return;
    };
    if !action_state.just_pressed(&Action::Action1) {
        return;
    }
    let Ok(craft_menu) = craft_menu_query.get_single() else {
        return;
    };
    if selected_recipe.0 >= craft_menu.0.craftable_recipes.len() {
        return;
    }
    let (recipe, selection) = &craft_menu.0.craftable_recipes[selected_recipe.0];
    let Ok(mut hotbar) = hotbar_query.get_single_mut() else {
        return;
    };
    for (slot, qty) in selection.iter() {
        let _ = hotbar.0.0[*slot].take(*qty);
    }
    hotbar.0.try_add(Stack::Some(recipe.out.0, recipe.out.1));
}