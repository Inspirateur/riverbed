use std::fs;
use bevy::{prelude::*, color::palettes::css};
use leafwing_input_manager::action_state::ActionState;
use crate::{agents::{Action, PlayerControlled, HOTBAR_SLOTS}, items::{new_inventory, parse_recipes, CraftEntry, InventoryTrait, InventoryRecipes, Item, Recipe, Stack}, sounds::ItemGet};
use super::{game_menu::despawn_screen, ui_tex_map::{UiSlotKind, UiTextureMap}, GameUiState, ItemHolder, UIAction};

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
struct HandCrafts(Vec<CraftEntry>);

#[derive(Component)]
struct RecipeSlot(pub usize);

#[derive(Resource)]
pub struct SelectedRecipe(pub usize);

#[derive(Component)]
struct CraftingMenu(pub InventoryRecipes);

fn add_ingredient(parent: &mut ChildSpawnerCommands, item: &Item, qty: u32, is_craftable: bool, tex_map: &Res<UiTextureMap>) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Column,
        margin: UiRect::all(Val::Vw(0.2)),
        ..Default::default()
    }).with_children(|node| {
        tex_map.make_item_slot(
            node, &Stack::Some(*item, qty), 
            if is_craftable { UiSlotKind::Default } else { UiSlotKind::Disabled }
        );
    });
}

fn add_recipe_node(parent: &mut ChildSpawnerCommands, recipe: &Recipe, is_craftable: bool, tex_map: &Res<UiTextureMap>, slot: usize) {
    parent.spawn(Node {
        padding: UiRect::all(Val::Percent(0.4)), 
        width: Val::Percent(100.),
        justify_content: JustifyContent::End,
        ..Default::default()
    })
    .insert(RecipeSlot(slot))
    .with_children(|node| {
        for (ingredient, qty) in &recipe.ingredients {
            add_ingredient(node, ingredient, *qty, is_craftable, tex_map);
        }
        node.spawn((
            Text::new("=>"),
            TextFont {
                font_size: 40.,
                ..Default::default()
            },
            TextColor(if is_craftable {
                Color::Srgba(css::WHITE)
            } else {
                Color::Srgba(css::GRAY)
            }),
            Node {
                margin: UiRect::horizontal(Val::Vw(0.1)),
                ..Default::default()
            }
        ));
        add_ingredient(node, &recipe.out.0, recipe.out.1, is_craftable, tex_map);
    });
}

fn create_craft_menu(mut commands: Commands, inventory_recipes: InventoryRecipes, tex_map: Res<UiTextureMap>) {
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(25.),
            height: Val::Percent(80.),
            left: Val::VMin(5.),
            top: Val::VMin(5.),
            ..Default::default()
        },
        BackgroundColor(Color::LinearRgba(LinearRgba::new(0., 0., 0., 0.9)))
    ))
    .with_children(
        |parent| {
            parent.spawn((
                Text::new("Craft recipes"),
                TextFont {
                    font_size: 40.,
                    ..Default::default()
                },
                Node {
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
            ));
            let mut i = 0;
            for (craftable_recipe, _) in &inventory_recipes.craftable_recipes {
                add_recipe_node(parent, craftable_recipe, true, &tex_map, i);
                i += 1;
            }

            for uncraftable_recipe in &inventory_recipes.uncraftable_entries {
                add_recipe_node(parent, uncraftable_recipe.get_example(0), false, &tex_map, i);
                i += 1;
            }
    })
    .insert(CraftingMenu(inventory_recipes));
}

fn open_craft_menu(
    commands: Commands,
    handcraft_recipes: Res<HandCrafts>,
    hotbar_query: Query<&ItemHolder, With<PlayerControlled>>,
    tex_map: Res<UiTextureMap>,
) {
    let hotbar = match hotbar_query.single() {
        Ok(ItemHolder::Inventory(hotbar)) => hotbar,
        _ => &new_inventory::<HOTBAR_SLOTS>(),
    };
    let inventory_recipes = hotbar.filter_recipes(&handcraft_recipes.0);
    create_craft_menu(commands, inventory_recipes, tex_map);
}

fn refresh_craft_menu(
    mut commands: Commands,
    handcraft_recipes: Res<HandCrafts>,
    hotbar_query: Query<&ItemHolder, (With<PlayerControlled>, Changed<ItemHolder>)>,
    tex_map: Res<UiTextureMap>,
    craft_menu_query: Query<Entity, With<CraftingMenu>>,
) {
    let Ok(ItemHolder::Inventory(hotbar)) = hotbar_query.single() else {
        return;
    };
    if let Ok(entity) = craft_menu_query.single() {
        commands.entity(entity).despawn();
    };
    let inventory_recipes = hotbar.filter_recipes(&handcraft_recipes.0);
    create_craft_menu(commands, inventory_recipes, tex_map);
}

fn display_selected_recipe(
    selected_recipe: Res<SelectedRecipe>,
    mut recipe_query: Query<(&RecipeSlot, &mut BackgroundColor)>,
    craft_menu_query: Query<&CraftingMenu>,
) {
    let Ok(craft_menu) = craft_menu_query.single() else {
        return;
    };
    let slots = craft_menu.0.craftable_recipes.len();
    for (slot, mut bg) in recipe_query.iter_mut() {
        if selected_recipe.0 < slots && slot.0 == selected_recipe.0 {
            bg.0 = Color::linear_rgba(1., 1., 1., 0.2);
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
    let Ok(craft_menu) = craft_menu_query.single() else {
        return;
    };
    let slots = craft_menu.0.craftable_recipes.len();
    if slots == 0 {
        return;
    }
    let Ok(action_state) = action_query.single() else {
        return;
    };
    if action_state.pressed(&UIAction::ScrollUp) {
        selected_recipe.0 = (selected_recipe.0 + slots - 1).rem_euclid(slots);
    } else if action_state.pressed(&UIAction::ScrollDown) {
        selected_recipe.0 = (selected_recipe.0 + 1).rem_euclid(slots);
    }
}

fn craft_action(
    mut commands: Commands,
    selected_recipe: Res<SelectedRecipe>,
    craft_menu_query: Query<&CraftingMenu>,
    mut hotbar_query: Query<(Entity, &mut ItemHolder), With<PlayerControlled>>,
    action_query: Query<&ActionState<Action>>,
) {
    let Ok(action_state) = action_query.single() else {
        return;
    };
    if !action_state.just_pressed(&Action::Hit) {
        return;
    }
    let Ok(craft_menu) = craft_menu_query.single() else {
        return;
    };
    if selected_recipe.0 >= craft_menu.0.craftable_recipes.len() {
        return;
    }
    let (recipe, selection) = &craft_menu.0.craftable_recipes[selected_recipe.0];
    let Ok((player, mut hotbar)) = hotbar_query.single_mut() else {
        return;
    };
    let ItemHolder::Inventory(ref mut hotbar) = *hotbar else {
        return;
    };
    for (slot, qty) in selection.iter() {
        let _ = hotbar[*slot].take(*qty);
    }
    hotbar.try_add(Stack::Some(recipe.out.0, recipe.out.1));
    commands.trigger_targets(ItemGet, player);
}
