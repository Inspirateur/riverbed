use std::collections::HashMap;
use bevy::prelude::Component;

use super::{craft_table::Recipe, item::Item, CraftEntry};
pub const HOTBAR_SLOTS: usize = 8;

#[derive(Default, Debug)]
pub enum Stack {
    Some(Item, u32),
    #[default]
    None
}


impl Stack {
    pub fn try_add(&mut self, other: Stack) -> Option<Stack> {
        let Stack::Some(other_item, other_stack) = other else {
            return None;
        };

        if let Stack::Some(item, stack) = self {
            if *item == other_item {
                *stack += other_stack;
                None
            } else {
                Some(other)
            }
        } else {
            *self = other;
            None
        }
    }

    pub fn item(&self) -> Option<&Item> {
        match self {
            Stack::None => None,
            Stack::Some(item, _) => Some(item)
        }
    }

    pub fn quantity(&self) -> u32 {
        match self {
            Stack::Some(_, n) => *n,
            Stack::None => 0
        }
    }

    pub fn take(&mut self, n: u32) -> Stack {
        if n == 0 {
            return Stack::None;
        }
        let Stack::Some(item, amount) = self else {
            return Stack::None;
        };
        let n = n.min(*amount);
        let res = Stack::Some(item.clone(), n);
        *amount -= n;
        if *amount == 0 {
            *self = Stack::None;
        }
        res
    }
}

pub struct InventoryRecipes {
    pub craftable_recipes: Vec<(Recipe, HashMap<usize, u32>)>,
    pub uncraftable_entries: Vec<CraftEntry>
}

pub struct Inventory<const N: usize>(pub [Stack; N]);


impl<const N: usize> Inventory<N> {
    pub fn new() -> Self {
        Self(std::array::from_fn(|_| Stack::None))
    }

    pub fn try_add(&mut self, mut stack: Stack) -> Option<Stack> {
        // try to add to an existing stack first
        for i in 0..self.0.len() {
            if matches!(self.0[i], Stack::None) {
                continue;
            }
            stack = self.0[i].try_add(stack)?;
        }
        // if not possible, just add to the first empty slot
        for i in 0..self.0.len() {
            stack = self.0[i].try_add(stack)?;
        }
        Some(stack)
    }

    fn try_select_item(&self, target_item: &Item, mut target_quantity: u32, selection: &mut HashMap<usize, u32>) -> bool {
        for (i, stack) in self.0.iter().enumerate() {
            let Stack::Some(item, mut qty) = stack else {
                continue;
            };
            if item != target_item {
                continue;
            }
            qty = (qty - selection.get(&i).unwrap_or(&0)).min(target_quantity);
            *selection.entry(i).or_insert(0) += qty;
            target_quantity -= qty;
            if target_quantity == 0 {
                return true;
            }
        }
        false
    }

    fn is_recipe_craftable(&self, recipe: &Recipe) -> Option<HashMap<usize, u32>> {
        let mut selection: HashMap<usize, u32> = HashMap::new();
        for (ingredient, qty) in &recipe.ingredients {
            if !self.try_select_item(ingredient, *qty, &mut selection) {
                return None;
            }
        }
        Some(selection)
    }

    pub fn filter_recipes(&self, recipes: &Vec<CraftEntry>) -> InventoryRecipes {
        // Only return recipes that are possible to make with this inventory
        let mut craftable_recipes = Vec::new();
        let mut uncraftable_recipes = Vec::new();
        for craft_entry in recipes.iter() {
            match craft_entry {
                CraftEntry::RecipeGroup(recipes) => {
                    let mut got_any = false;
                    for recipe in recipes {
                        if let Some(selection) = self.is_recipe_craftable(recipe) {
                            craftable_recipes.push((recipe.clone(), selection));
                            got_any = true;
                        }
                    }
                    if !got_any {
                        uncraftable_recipes.push(craft_entry.clone());
                    }
                },
                CraftEntry::Recipe(recipe) => {
                    if let Some(selection) = self.is_recipe_craftable(recipe) {
                        craftable_recipes.push((recipe.clone(), selection));
                    } else {
                        uncraftable_recipes.push(craft_entry.clone());
                    }
                },
            }
            
        }
        InventoryRecipes { craftable_recipes, uncraftable_entries: uncraftable_recipes }
    }
}

#[derive(Component)]
pub struct Hotbar(pub Inventory<HOTBAR_SLOTS>);


#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{items::{craft_table::Recipe, parse_recipes, Item}, Block};
    use super::{Inventory, Stack};

    #[test]
    fn test_recipe_filter() {
        let mut inventory: Inventory<8> = Inventory::new();
        inventory.0[0] = Stack::Some(Item::Block(Block::OakLog), 2);
        inventory.0[1] = Stack::Some(Item::Rock, 2);
        inventory.0[3] = Stack::Some(Item::Stick, 10);
        inventory.0[4] = Stack::Some(Item::Rock, 1);
        inventory.0[6] = Stack::Some(Item::Block(Block::Dirt), 1);
        inventory.0[7] = Stack::Some(Item::Block(Block::BirchLog), 4);
        let recipes_str = r#"
        {Soil} + Dirt = Mud
        3 Rock + Stick = StoneAxe
        4 Rock + Stick = StoneAxe
        5 {Wood}Log = Campfire
        "#;
        let recipes = parse_recipes(recipes_str);
        let available_recipes = vec![
            Recipe {
                ingredients: vec![(Item::Rock, 3), (Item::Stick, 1)],
                out: (Item::StoneAxe, 1)
            }, 
            Recipe {
                ingredients: vec![(Item::Block(Block::BirchLog), 5)],
                out: (Item::Block(Block::Campfire), 1)
            }
        ];
        assert_eq!(
            available_recipes, 
            inventory.filter_recipes(&recipes).craftable_recipes.into_iter().map(|(recipe, _)| recipe).collect_vec()
        );
    }
}