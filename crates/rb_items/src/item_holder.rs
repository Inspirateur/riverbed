use crate::inventory::{InventoryTrait, Stack};
use crate::item::Item;
use bevy::prelude::Component;

// TODO: If/When trait queries get adopted by Bevy (https://github.com/bevyengine/bevy/issues/15970)
// get rid of this enum and use a trait instead, item holding components will implement this trait
#[derive(Component)]
pub enum ItemHolder {
    Furnace {
        fuel: Stack,
        material: Stack,
        output: Stack,
    },
    Inventory(Box<[Stack]>),
}

pub enum FurnaceSlot {
    Material,
    Fuel,
    Output,
}

impl From<FurnaceSlot> for usize {
    fn from(value: FurnaceSlot) -> Self {
        value as usize
    }
}

impl From<usize> for FurnaceSlot {
    fn from(value: usize) -> Self {
        match value {
            0 => FurnaceSlot::Material,
            1 => FurnaceSlot::Fuel,
            2 => FurnaceSlot::Output,
            _ => unreachable!(),
        }
    }
}

/// Convenience constructor for a freshly created furnace's item slots.
pub fn furnace_slots() -> ItemHolder {
    ItemHolder::Furnace {
        fuel: Stack::None,
        material: Stack::None,
        output: Stack::None,
    }
}

impl ItemHolder {
    fn can_receive(&self, item: &Item, slot_id: usize) -> bool {
        match self {
            ItemHolder::Furnace { .. } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => true,
                    FurnaceSlot::Fuel => item == &Item::Coal,
                    FurnaceSlot::Output => false,
                }
            }
            ItemHolder::Inventory(_) => true,
        }
    }

    pub fn get(&self, slot_id: usize) -> &Stack {
        match self {
            ItemHolder::Furnace {
                fuel,
                material,
                output,
            } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => material,
                    FurnaceSlot::Fuel => fuel,
                    FurnaceSlot::Output => output,
                }
            }
            ItemHolder::Inventory(vec) => &vec[slot_id],
        }
    }

    pub fn get_mut(&mut self, slot_id: usize) -> &mut Stack {
        match self {
            ItemHolder::Furnace {
                fuel,
                material,
                output,
            } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => material,
                    FurnaceSlot::Fuel => fuel,
                    FurnaceSlot::Output => output,
                }
            }
            ItemHolder::Inventory(vec) => &mut vec[slot_id],
        }
    }

    pub fn try_add(&mut self, stack: Stack) -> Option<Stack> {
        match self {
            ItemHolder::Furnace {
                fuel: _,
                material: _,
                output: _,
            } => todo!(),
            ItemHolder::Inventory(items) => items.try_add(stack),
        }
    }
}
