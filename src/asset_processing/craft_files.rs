use std::{collections::HashMap, fmt::Debug};
use itertools::Itertools;
use regex::Regex;
use strum::IntoEnumIterator;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_CRAFT: Regex = Regex::new(r"\[([^\]]+)\]").unwrap();
}

#[derive(Debug)]
pub struct RecipeExpander(HashMap<String, Vec<String>>);

impl RecipeExpander {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn register_enum<E>(&mut self) 
        where E: IntoEnumIterator + Debug
    {
        let variants = E::iter().map(|v| format!("{v:?}")).collect();
        // This is kind of smelly but it's also easy to get rid of (by accepting a parameter for the type name)
        self.0.insert(std::any::type_name::<E>().split("::").last().unwrap().to_string(), variants);
    }

    pub fn try_expand(&self, recipe: &str) -> Option<Vec<String>> {
        let Some(captures) = RE_CRAFT.captures(recipe) else {
            return None;
        };
        let groups = captures.iter().skip(1).flatten().map(|c| c.as_str()).unique().collect::<Vec<_>>();
        let mut res = Vec::new();
        for variants in groups.iter().map(|g| self.0.get(*g).unwrap()).multi_cartesian_product() {
            for (group, variant) in groups.iter().zip(variants) {
                res.push(recipe.replace(&format!("[{}]", *group), variant));
            }
        }
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    use strum_macros::EnumIter;

    use super::RecipeExpander;

    #[derive(Debug, EnumIter)]
    enum Wood {
        Acacia,
        Birch,
        Oak,
        Spruce
    }

    #[test]
    fn test_expand() {
        let mut expander = RecipeExpander::new();
        expander.register_enum::<Wood>();
        let recipe_group = expander.try_expand("[Wood]Log = 4 [Wood]Plank");
        println!("{recipe_group:?}");
    }
}