use std::{collections::HashMap, hash::Hash, ops::Index, slice::Iter};

#[derive(Debug)]
pub struct Palette<E: Hash> {
    leftmap: HashMap<E, usize>,
    rightmap: Vec<E>
}

impl<E: Hash> Palette<E> {
    pub fn iter(&self) -> Iter<'_, E> {
        self.rightmap.iter()
    }
}

impl<E: Hash + Eq + PartialEq + Clone> Palette<E> {
    pub fn new() -> Self {
        Self { leftmap: HashMap::new(), rightmap: Vec::new() }
    }

    pub fn index(&mut self, elem: E) -> usize {
        *self.leftmap.entry(elem.clone()).or_insert_with(|| {
            self.rightmap.push(elem);
            self.rightmap.len()-1
        })
    }

    pub fn map_to(&self, other: &Palette<E>) -> Vec<Option<usize>> {
        self.rightmap.iter().map(|e| other.leftmap.get(e).cloned()).collect()
    }
}

impl<E: Hash> Index<usize> for Palette<E> {
    type Output = E;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rightmap[index]
    }
}