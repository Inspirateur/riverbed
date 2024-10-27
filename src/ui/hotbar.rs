use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use crate::agents::{PlayerControlled, PlayerSpawn, HOTBAR_SLOTS};
use super::{ui_tex_map::{UiSlotKind, UiTextureMap}, ControllingPlayer, UIAction, UISlot};
const SLOT_SIZE_PERCENT: f32 = 4.5;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedHotbarSlot(0))
            .add_systems(Startup, setup_hotbar_display.after(PlayerSpawn))
            .add_systems(Update, scroll_hotbar.run_if(in_state(ControllingPlayer)))
            ;
    }
}

#[derive(Component)]
struct HotBarSlot;

#[derive(Resource)]
pub struct SelectedHotbarSlot(pub usize);

fn setup_hotbar_display(
    mut commands: Commands, player_query: Query<Entity, With<PlayerControlled>>
) {
    let Ok(player_entt) = player_query.get_single() else {
        return;
    };
    for i in 0..HOTBAR_SLOTS {
        let right_border = if i < HOTBAR_SLOTS - 1 { Val::Px(0.) } else { Val::Px(4.) };
        let left_offset = Val::Percent(50.+(i as f32-HOTBAR_SLOTS as f32/2.)*SLOT_SIZE_PERCENT);
        let img_style = Style {
            position_type: PositionType::Absolute,
            left: left_offset,
            bottom: Val::Percent(SLOT_SIZE_PERCENT),
            width: Val::Percent(SLOT_SIZE_PERCENT),
            aspect_ratio: Some(1.),
            border: UiRect::new(Val::Px(4.), right_border, Val::Px(4.), Val::Px(4.)),
            ..Default::default()
        };
        commands.spawn(NodeBundle { style: img_style, ..Default::default()})
            .insert(Interaction::default())
            .insert(UISlot(player_entt, i))
            .insert(HotBarSlot)
            .with_children(|node|
                UiTextureMap::make_empty_item_slot(node, UiSlotKind::Default)
            );
    }
}

fn scroll_hotbar(
    mut selected_slot: ResMut<SelectedHotbarSlot>, 
    action_query: Query<&ActionState<UIAction>>,
    node_query: Query<(&UISlot, &Children), With<HotBarSlot>>,
    mut img_query: Query<&mut UiImage>,
    mut text_query: Query<&mut Text>,
    mut bg_query: Query<&mut BackgroundColor, With<UiImage>>,
) {
    let Ok(action_state) = action_query.get_single() else {
        return;
    };
    if action_state.pressed(&UIAction::ScrollUp) {
        selected_slot.0 = (selected_slot.0 as i32 - 1).rem_euclid(HOTBAR_SLOTS as i32) as usize;
    } else if action_state.pressed(&UIAction::ScrollDown) {
        selected_slot.0 = (selected_slot.0 + 1).rem_euclid(HOTBAR_SLOTS);
    }
    for (UISlot(_, slot), children) in node_query.iter() {
        let selected = *slot == selected_slot.0;
        for child in children {
            if let Ok(mut ui_img) = img_query.get_mut(*child) {
                ui_img.color.set_alpha(if selected { 1. } else { 0.75 });
            }
            if let Ok(mut text) = text_query.get_mut(*child) {
                text.sections[0].style.color = if selected {
                    Color::linear_rgba(1., 1., 1., 1.)
                } else {
                    Color::linear_rgba(0.6, 0.6, 0.6, 1.)
                }
            }
            if let Ok(mut bg) = bg_query.get_mut(*child) {
                bg.0 = if selected {
                    Color::linear_rgba(0., 0., 0., 0.8)
                } else {
                    Color::linear_rgba(0., 0., 0., 0.5)
                }
            }
        }
    }
}
