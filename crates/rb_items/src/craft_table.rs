use std::{fmt::Debug, str::FromStr};
use itertools::Itertools;
use rb_asset_processing::RecipeExpander;
use rb_block::{Soil, Wood};
use crate::item::Item;

#[derive(Debug, Clone)]
pub enum CraftEntry {
    RecipeGroup(Vec<Recipe>),
    Recipe(Recipe)
}

impl CraftEntry {
    pub fn get_example(&self, seed: u32) -> &Recipe {
        match self {
            CraftEntry::RecipeGroup(recipes) => &recipes[seed as usize%recipes.len()],
            CraftEntry::Recipe(recipe) => recipe,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Recipe {
    pub ingredients: Vec<(Item, u32)>,
    pub out: (Item, u32),
}

fn parse_ingredient_qty<T>(ingredient: &str) -> (T, u32) 
where 
    T: FromStr,
    T::Err: Debug
{
    let mut parts = ingredient.split_whitespace();
    let qt_or_item = parts.next().unwrap();
    if let Some(item) = parts.next() {
        (T::from_str(item).unwrap(), qt_or_item.parse().unwrap())
    } else {
        (T::from_str(qt_or_item).unwrap(), 1)
    }
}

pub fn parse_recipe<S: AsRef<str>>(recipe: S) -> Recipe {
    let recipe: &str = recipe.as_ref();
    let (ingredients, out) = recipe.split("=").next_tuple().unwrap();
    let ingredients = ingredients.split("+").map(parse_ingredient_qty).collect_vec();
    let out = parse_ingredient_qty(out);
    Recipe { ingredients, out }
}

pub fn parse_recipes(recipes: &str) -> Vec<CraftEntry> {
    let mut expander = RecipeExpander::new();
    expander.register_enum::<Wood>();
    expander.register_enum::<Soil>();    
    recipes.trim().lines().map(|recipe| {
        match expander.try_expand(recipe) {
            None => CraftEntry::Recipe(parse_recipe(recipe)),
            Some(recipe_group) => CraftEntry::RecipeGroup(recipe_group.into_iter().map(parse_recipe).collect())
        }
    }).collect_vec()
}
