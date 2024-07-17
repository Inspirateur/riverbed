use std::collections::HashMap;
use bevy::prelude::Component;
use itertools::Itertools;
use super::{craft_table::{Ingredient, Recipe}, item::Item};
pub const HOTBAR_SLOTS: usize = 8;

pub enum Stack {
    Some(Item, u32),
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

pub struct Inventory<const N: usize>(pub [Stack; N]);


impl<const N: usize> Inventory<N> {
    pub fn new() -> Self {
        Self(std::array::from_fn(|_| Stack::None))
    }

    pub fn try_add(&mut self, mut stack: Stack) -> Option<Stack> {
        for i in 0..self.0.len() {
            stack = self.0[i].try_add(stack)?;
        }
        Some(stack)
    }

    fn contents(&self) -> HashMap<Item, u32> {
        let mut res = HashMap::new();
        for slot in &self.0 {
            if let Stack::Some(item, qty) = slot {
                *res.entry(item.clone()).or_insert(0) += qty;
            }
        }
        res
    }

    pub fn filter_recipes(&self, recipes: &Vec<Recipe>) -> Vec<Recipe> {
        // Only return recipes that are possible to make with this inventory
        let contents = self.contents();
        let mut used: HashMap<Item, u32> = HashMap::new();
        recipes.iter().filter(|recipe| {
            used.clear();
            // go through the specific ingredients first
            for (ingredient, qty) in &recipe.ingredients {
                let Ingredient::Item(item) = ingredient else {
                    continue;
                };
                if contents.get(&item).unwrap_or(&0) < &qty {
                    return false;
                }
                used.insert(*item, *qty);
            }
            // go through the block families second
            for (ingredient, mut qty) in &recipe.ingredients {
                let Ingredient::BlockFamily(family) = ingredient else {
                    continue;
                };
                for (available_item, available_qty) in contents.iter() {
                    let available_qty = available_qty - used.get(&available_item).unwrap_or(&0);
                    let Item::Block(block) = available_item else {
                        continue;
                    };
                    if !block.families().contains(family) {
                        continue;
                    }
                    qty -= available_qty.min(qty);
                    if qty == 0 {
                        break;
                    }
                }
                if qty > 0 {
                    return false;
                }
            }
            true
        }).cloned().collect_vec()
    }
}

#[derive(Component)]
pub struct Hotbar(pub Inventory<HOTBAR_SLOTS>);


#[cfg(test)]
mod tests {
    use crate::{blocks::{Block, BlockFamily}, items::{craft_table::{Ingredient, Recipe}, Item}};
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
        let recipes = vec![
            // NO: we got only 1 Dirt (we need 1 Dirt + 1 Soil)
            Recipe {
                ingredients: vec![
                    (Ingredient::BlockFamily(BlockFamily::Soil), 1),
                    (Ingredient::Item(Item::Block(Block::Dirt)), 1),
                ],
                out: (Item::Block(Block::Mud), 1)
            },
            // YES: we got 3 rocks total
            Recipe {
                ingredients: vec![
                    (Ingredient::Item(Item::Rock), 3),
                    (Ingredient::Item(Item::Stick), 1),
                ],
                out: (Item::StoneAxe, 1)
            },
            // NO: we don't have enough rocks
            Recipe {
                ingredients: vec![
                    (Ingredient::Item(Item::Rock), 4),
                    (Ingredient::Item(Item::Stick), 1),
                ],
                out: (Item::StoneAxe, 1)
            },
            // YES: we got 6 logs total
            Recipe {
                ingredients: vec![
                    (Ingredient::BlockFamily(BlockFamily::Log), 5),
                ],
                out: (Item::Block(Block::Campfire), 1)
            },
        ];
        let available_recipes = vec![recipes[1].clone(), recipes[3].clone()];
        assert_eq!(available_recipes, inventory.filter_recipes(&recipes));
    }
}