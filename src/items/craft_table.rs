use std::{fmt::Debug, str::FromStr};
use itertools::Itertools;
use serde::Deserialize;
use crate::blocks::BlockFamily;
use super::Item;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(untagged)]
pub enum Ingredient {
    Item(Item),
    BlockFamily(BlockFamily),
}

impl FromStr for Ingredient {
    type Err = json5::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        json5::from_str(&format!("'{}'", s))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Recipe {
    pub ingredients: Vec<(Ingredient, u32)>,
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

pub fn parse_recipes(recipes: &str) -> Vec<Recipe> {
    recipes.trim().lines().map(|recipe| {
        let (ingredients, out) = recipe.split("=").next_tuple().unwrap();
        let ingredients = ingredients.split("+").map(parse_ingredient_qty).collect_vec();
        let out = parse_ingredient_qty(out);
        Recipe { ingredients, out }
    }).collect_vec()
}


#[cfg(test)]
mod tests {
    use crate::blocks::Block;
    use super::*;

    #[test]
    fn test_parsing() {
        let recipes_str = r#"
        Log + 4 Rock = Campfire
        2 Soil + SeaBlock + Dirt = 3 Mud
        "#;
        let recipes = parse_recipes(recipes_str);
        assert_eq!(recipes[0], Recipe { 
            ingredients: vec![
                (Ingredient::BlockFamily(BlockFamily::Log), 1),
                (Ingredient::Item(Item::Rock), 4)
            ],
            out: (Item::Block(Block::Campfire), 1)
        });
        assert_eq!(recipes[1], Recipe { 
            ingredients: vec![
                (Ingredient::BlockFamily(BlockFamily::Soil), 2),
                (Ingredient::Item(Item::Block(Block::SeaBlock)), 1),
                (Ingredient::Item(Item::Block(Block::Dirt)), 1)
            ],
            out: (Item::Block(Block::Mud), 3)
        });
    }
}