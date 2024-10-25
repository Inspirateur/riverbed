use std::collections::HashMap;
use super::{craft_table::Recipe, item::Item, CraftEntry};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Stack {
    Some(Item, u32),
    #[default]
    None
}


impl Stack {
    pub fn can_add(&self, other: Stack) -> bool {
        let Stack::Some(item, _) = self else {
            return true;
        };
        let Stack::Some(other_item, _) = other else {
            return true;
        };
        item == &other_item
    }
    
    /// Tries to add other to self, and output what couldn't be added (either None or other in the case of uncapped stacks)
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

    /// Tries to take from other to self, outputs true if it could take at least 1 item, false otherwise
    pub fn try_take_from(&mut self, other: &mut Stack) -> bool {
        match self.try_add(other.clone()) {
            Some(remainder) => {
                if remainder == *other {
                    // nothing could be taken
                    false
                } else {
                    *other = remainder;
                    true
                }
            },
            None => {
                *other = Stack::None;
                true
            },
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
    
    /// Intentionnaly private so we don't clone Stacks
    fn clone(&self) -> Stack {
        match self {
            Stack::Some(item, qty) => Stack::Some(item.clone(), *qty),
            Stack::None => Stack::None,
        }
    }

    pub fn swap_with(&mut self, other: &mut Stack) {
        let self_clone = self.clone();
        *self = other.clone();
        *other = self_clone;
    }
}

pub struct InventoryRecipes {
    pub craftable_recipes: Vec<(Recipe, HashMap<usize, u32>)>,
    pub uncraftable_entries: Vec<CraftEntry>
}

pub trait InventoryTrait {
    fn try_add(&mut self, stack: Stack) -> Option<Stack>;

    fn try_select_item(&self, target_item: &Item, target_quantity: u32, selection: &mut HashMap<usize, u32>) -> bool;

    fn is_recipe_craftable(&self, recipe: &Recipe) -> Option<HashMap<usize, u32>>;

    fn filter_recipes(&self, recipes: &Vec<CraftEntry>) -> InventoryRecipes;    
}


impl InventoryTrait for [Stack] {
    fn try_add(&mut self, mut stack: Stack) -> Option<Stack> {
        // try to add to an existing stack first
        for i in 0..self.len() {
            if matches!(self[i], Stack::None) {
                continue;
            }
            stack = self[i].try_add(stack)?;
        }
        // if not possible, just add to the first empty slot
        for i in 0..self.len() {
            stack = self[i].try_add(stack)?;
        }
        Some(stack)
    }

    fn try_select_item(&self, target_item: &Item, mut target_quantity: u32, selection: &mut HashMap<usize, u32>) -> bool {
        for (i, stack) in self.iter().enumerate() {
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

    fn filter_recipes(&self, recipes: &Vec<CraftEntry>) -> InventoryRecipes {
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

pub fn new_inventory<const N: usize>() -> Box<[Stack]> {
    Box::<[Stack; N]>::new(core::array::from_fn(|_| Stack::None))
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{items::{craft_table::Recipe, new_inventory, parse_recipes, Item}, Block};
    use super::{InventoryTrait, Stack};

    #[test]
    fn test_recipe_filter() {
        let mut inventory = new_inventory::<8>();
        inventory[0] = Stack::Some(Item::Block(Block::OakLog), 2);
        inventory[1] = Stack::Some(Item::Rock, 2);
        inventory[3] = Stack::Some(Item::Stick, 10);
        inventory[4] = Stack::Some(Item::Rock, 1);
        inventory[6] = Stack::Some(Item::Block(Block::Dirt), 1);
        inventory[7] = Stack::Some(Item::Block(Block::BirchLog), 4);
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