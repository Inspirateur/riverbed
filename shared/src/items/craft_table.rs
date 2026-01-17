use std::{fmt::Debug, str::FromStr};
use itertools::Itertools;
use crate::{asset_processing::RecipeExpander, Soil, Wood};
use super::Item;

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


#[cfg(test)]
mod tests {
    use crate::Block;
    use super::*;

    #[test]
    fn test_parsing() {
        let recipes_str = r#"
        {Wood}Log + 4 Rock = Campfire
        2 {Soil} + SeaBlock + Dirt = 3 Mud
        "#;
        let recipes = parse_recipes(recipes_str);
        let CraftEntry::RecipeGroup(group1) = &recipes[0] else {
            panic!("Craft entry 0 should be a RecipeGroup");
        };
        let CraftEntry::RecipeGroup(group2) = &recipes[1] else {
            panic!("Craft entry 1 should be a RecipeGroup");
        };
        assert_eq!(group1[0], Recipe { 
            ingredients: vec![
                (Item::Block(Block::AcaciaLog), 1),
                (Item::Rock, 4)
            ],
            out: (Item::Block(Block::Campfire), 1)
        });
        assert_eq!(group2[0], Recipe { 
            ingredients: vec![
                (Item::Block(Block::CoarseDirt), 2),
                (Item::Block(Block::SeaBlock), 1),
                (Item::Block(Block::Dirt), 1)
            ],
            out: (Item::Block(Block::Mud), 3)
        });
    }
}