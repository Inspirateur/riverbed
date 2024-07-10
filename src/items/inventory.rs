use bevy::prelude::Component;
use super::item::Item;
pub const HOTBAR_SLOTS: usize = 8;

pub enum Stack {
    Some(Item, usize),
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

    pub fn quantity(&self) -> usize {
        match self {
            Stack::Some(_, n) => *n,
            Stack::None => 0
        }
    }

    pub fn take(&mut self, n: usize) -> Stack {
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
}

#[derive(Component)]
pub struct Hotbar(pub Inventory<HOTBAR_SLOTS>);
