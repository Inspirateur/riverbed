use bevy::prelude::Component;
use super::item::Item;

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

    pub fn quantity(&self) -> usize {
        match self {
            Stack::Some(_, n) => *n,
            Stack::None => 0
        }
    }

    pub fn take(&mut self, n: usize) -> Stack {
        match self {
            Stack::Some(item, _n) => {
                let n = n.min(*_n);
                *_n -= n;
                Stack::Some(item.clone(), n)
            },
            Stack::None => Stack::None
        }
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
pub struct Hotbar(pub Inventory<8>);
