use bevy::{color::palettes::css, prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE};
use leafwing_input_manager::prelude::*;
use crate::{
    agents::PlayerControlled, 
    items::{Hotbar, Item, Stack, HOTBAR_SLOTS}, 
};
use super::{ui_tex_map::UiTextureMap, ControllingPlayer, UIAction};
const SLOT_SIZE_PERCENT: f32 = 4.5;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedHotbarSlot(0))
            .add_systems(Startup, setup_hotbar_display)
            .add_systems(Update, display_hotbar)
            .add_systems(Update, scroll_hotbar.run_if(in_state(ControllingPlayer)))
            ;
    }
}

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Resource)]
pub struct SelectedHotbarSlot(pub usize);

fn setup_hotbar_display(
    mut commands: Commands, asset_server: Res<AssetServer>
) {
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
        commands.spawn(ImageBundle { style: img_style, ..Default::default()} )
            .insert(HotbarSlot(i))
            .insert(BorderColor(Color::srgba(1., 1., 1., 1.)));

        let text_style = Style {
            position_type: PositionType::Absolute,
            left: left_offset,
            bottom: Val::Percent(SLOT_SIZE_PERCENT),
            margin: UiRect::new(Val::Px(4.), Val::Px(0.), Val::Px(0.), Val::Px(4.)),
            ..Default::default()
        };
        commands.spawn(TextBundle {
            style: text_style,
            text: Text {
                sections: vec![TextSection { value: String::new(), style: TextStyle { 
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"), 
                    font_size: 24., 
                    color: Color::Srgba(css::BEIGE) 
                }}],
                ..Default::default()
            },
            ..Default::default()
        }).insert(HotbarSlot(i));
    }
}

fn display_hotbar(
    mut bg_query: Query<(&mut BackgroundColor, &HotbarSlot)>, 
    mut img_query: Query<(&mut UiImage, &HotbarSlot)>, 
    mut text_query: Query<(&mut Text, &HotbarSlot)>,
    tex_map: Res<UiTextureMap>,
    selected_slot: Res<SelectedHotbarSlot>,
    hotbar_query: Query<&Hotbar, (With<PlayerControlled>, Changed<Hotbar>)>,
) {
    if let Ok(hotbar) = hotbar_query.get_single() {
        for (mut img, slot) in img_query.iter_mut() {
            *img = if let Stack::Some(item, _) = hotbar.0.0[slot.0] {
                if let Some(handle) = tex_map.0.get(&item) {
                    UiImage::new(handle.clone()).with_color({
                        match item {
                            Item::Block(block) if block.is_foliage() => Color::linear_rgba(0.3, 1.0, 0.1, 1.),
                            _ => Color::linear_rgba(1., 1., 1., 1.)
                        }
                    })
                } else {
                    UiImage::new(TRANSPARENT_IMAGE_HANDLE)
                }
            } else {
                UiImage::new(TRANSPARENT_IMAGE_HANDLE)
            };
        }
        for (mut text, slot) in text_query.iter_mut() {
            let quantity = hotbar.0.0[slot.0].quantity();
            text.sections[0].value = if quantity < 2 { String::new() } else { quantity.to_string() };
        }
    };
    // Highlight the selected slot with a darker bg color
    for (mut img, slot) in img_query.iter_mut() {
        if img.texture == TRANSPARENT_IMAGE_HANDLE {
            continue;
        }
        if slot.0 == selected_slot.0 {
            img.color.set_alpha(1.);
        } else {
            img.color.set_alpha(0.5);
        }
    }
    for (mut bg, slot) in bg_query.iter_mut() {
        bg.0 = if slot.0 == selected_slot.0 {
            Color::linear_rgba(0., 0., 0., 0.8)
        } else {
            Color::linear_rgba(0., 0., 0., 0.5)
        }
    }
}

fn scroll_hotbar(
    mut selected_slot: ResMut<SelectedHotbarSlot>, 
    query: Query<&ActionState<UIAction>>
) {
    let Ok(action_state) = query.get_single() else {
        return;
    };
    if action_state.pressed(&UIAction::ScrollUp) {
        selected_slot.0 = (selected_slot.0 as i32 - 1).rem_euclid(HOTBAR_SLOTS as i32) as usize;
    } else if action_state.pressed(&UIAction::ScrollDown) {
        selected_slot.0 = (selected_slot.0 + 1).rem_euclid(HOTBAR_SLOTS);
    }
}
