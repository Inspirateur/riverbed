use bevy::{prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE};
use crate::items::{Inventory, Item, Stack};
use super::CursorGrabbed;

pub struct ItemSlotPlugin;

impl Plugin for ItemSlotPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Dragging(None))
            .add_systems(Update, item_slot_click.run_if(not(in_state(CursorGrabbed))))
            ;
    }
}

#[derive(Resource)]
struct Dragging(pub Option<(Stack, Entity)>);

#[derive(Component, Clone)]
pub struct UISlot(pub Entity, pub usize);

pub enum FurnaceSlot {
    Material,
    Fuel,
    Output
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
            _ => unreachable!()
        }
    }
}

// TODO: If/When trait queries get adopted by Bevy (https://github.com/bevyengine/bevy/issues/15970)
// get rid of this enum and use a trait instead, item holding components will implement this trait
#[derive(Component)]
pub enum ItemHolder {
    Furnace { fuel: Stack, material: Stack, output: Stack },
    Inventory(Box<[Stack]>)
}

impl ItemHolder {
    fn can_recieve(&self, item: &Item, slot_id: usize) -> bool {
        match self {
            ItemHolder::Furnace { fuel: _, material: _, output: _ } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => true,
                    FurnaceSlot::Fuel => item == &Item::Coal,
                    FurnaceSlot::Output => false,
                }        
            },
            ItemHolder::Inventory(_) => true,
        }
    }

    pub fn get(&self, slot_id: usize) -> &Stack {
        match self {
            ItemHolder::Furnace { fuel, material, output } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => material,
                    FurnaceSlot::Fuel => fuel,
                    FurnaceSlot::Output => output,
                } 
            },
            ItemHolder::Inventory(vec) => &vec[slot_id],
        }
    }

    pub fn get_mut(&mut self, slot_id: usize) -> &mut Stack {
        match self {
            ItemHolder::Furnace { fuel, material, output } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => material,
                    FurnaceSlot::Fuel => fuel,
                    FurnaceSlot::Output => output,
                } 
            },
            ItemHolder::Inventory(vec) => &mut vec[slot_id],
        }
    }

    fn take_or_swap(&mut self, stack: &mut Stack, slot_id: usize) {
        let own_stack = self.get_mut(slot_id);
        if own_stack.try_take_from(stack) {
            // we succeeded in taking some items from stack
            return;
        };
        // we weren't able to add anything so we swap
        own_stack.swap_with(stack)
    }

    pub fn try_add(&mut self, stack: Stack) -> Option<Stack> {
        match self {
            ItemHolder::Furnace { fuel, material, output } => todo!(),
            ItemHolder::Inventory(items) => items.try_add(stack),
        }
    }
}

pub fn furnace_slots() -> ItemHolder {
    ItemHolder::Furnace { fuel: Stack::Some(Item::Coal, 6), material: Stack::Some(Item::IronOre, 5), output: Stack::Some(Item::IronIngot, 4) }
}

fn item_slot_click(
    mut interaction_query: Query<(&Interaction, &UISlot), Changed<Interaction>>,
    mut dragging: ResMut<Dragging>,
    mut item_holders: Query<&mut ItemHolder>,
) {
    for (interaction, UISlot(item_holder_entt, slot_id)) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(mut clicked_item_holder) = item_holders.get_mut(*item_holder_entt) else {
            continue;
        };
        match &mut dragging.0 {
            Some((dragged_items, _)) => {
                println!("end dragging");
                clicked_item_holder.take_or_swap(dragged_items, *slot_id);
                dragging.0 = None;
            },
            None => {
                println!("want to start dragging");
                if clicked_item_holder.get(*slot_id) == &Stack::None {
                    println!("nothing in slot {}", slot_id);
                    continue;
                }
                println!("start dragging");
                let stack = clicked_item_holder.get_mut(*slot_id);
                dragging.0 = Some((stack.take_all(), *item_holder_entt));
            },
        }
    }
}

fn item_slot_update(
    ui_slot_query: Query<&UISlot>
) {

}

fn display_dragged(
    dragging: Res<Dragging>,
) {

}