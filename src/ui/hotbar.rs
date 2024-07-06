use std::collections::HashMap;
use bevy::{color::palettes::css, prelude::*};
use leafwing_input_manager::prelude::*;
use crate::{agents::PlayerControlled, items::{Hotbar, Item, Stack, HOTBAR_SLOTS}};
const SLOT_SIZE_PERCENT: f32 = 6.;

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Resource)]
struct UITextureMap(HashMap<Item, Handle<Image>>);

#[derive(Resource)]
struct SelectedHotbarSlot(usize);

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum HotbarScroll {
    Up, 
    Down,
}

fn setup_hotbar_display(
    mut commands: Commands, asset_server: Res<AssetServer>
) {
    let mut input_map = InputMap::default();
    input_map
        .insert(HotbarScroll::Up, MouseScrollDirection::UP)
        .insert(HotbarScroll::Down, MouseScrollDirection::DOWN)
        ;
    commands.spawn(InputManagerBundle::with_map(input_map));
    for i in 0..HOTBAR_SLOTS {
        let right_border = if i < HOTBAR_SLOTS - 1 { Val::Px(0.) } else { Val::Px(4.) };
        let style = Style {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.+(i as f32-HOTBAR_SLOTS as f32/2.)*SLOT_SIZE_PERCENT),
            width: Val::Percent(SLOT_SIZE_PERCENT),
            bottom: Val::Percent(SLOT_SIZE_PERCENT),
            aspect_ratio: Some(1.),
            border: UiRect::new(Val::Px(4.), right_border, Val::Px(4.), Val::Px(4.)),
            ..Default::default()
        };
        commands.spawn(ImageBundle { style: style.clone(), ..Default::default()} )
        .insert(TextBundle {
            style,
            text: Text {
                sections: vec![TextSection { value: String::new(), style: TextStyle { 
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"), 
                    font_size: 24., 
                    color: Color::Srgba(css::BEIGE) 
                }}],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(HotbarSlot(i));
    }
}

fn update_hotbar(
    mut bg_query: Query<(&mut BackgroundColor, &HotbarSlot)>, 
    mut img_query: Query<(&mut UiImage, &HotbarSlot)>, 
    mut text_query: Query<(&mut Text, &HotbarSlot)>,
    tex_map: Res<UITextureMap>,
    selected_slot: Res<SelectedHotbarSlot>,
    hotbar_query: Query<&Hotbar, (With<PlayerControlled>, Changed<Hotbar>)>,
) {
    let Ok(hotbar) = hotbar_query.get_single() else {
        // The hotbar hasn't changed since last time
        return;
    };
    
    for (mut bg, slot) in bg_query.iter_mut() {
        // TODO: these are no longer transparent since migration to bevy 0.14 
        bg.0 = if slot.0 == selected_slot.0 {
            Color::srgba(0., 0., 0., 0.5)
        } else {
            Color::srgba(0., 0., 0., 0.3)
        };
    }
    for (mut img, slot) in img_query.iter_mut() {
        img.texture = if let Stack::Some(item, _) = hotbar.0.0[slot.0] {
            // tex_map.0.get(&item).unwrap().clone_weak()
            Handle::default()
        } else {
            Handle::default()
        };
    }
    for (mut text, slot) in text_query.iter_mut() {
        let quantity = hotbar.0.0[slot.0].quantity();
        text.sections[0].value = if quantity < 2 { String::new() } else { quantity.to_string() };
    }
}

fn scroll_hotbar(mut selected_slot: ResMut<SelectedHotbarSlot>, query: Query<&ActionState<HotbarScroll>>) {
    let Ok(action_state) = query.get_single() else {
        return;
    };
    // TODO: none of those are triggering...
    if action_state.pressed(&HotbarScroll::Up) {
        selected_slot.0 = (selected_slot.0 + 1) % HOTBAR_SLOTS;
    } else if action_state.pressed(&HotbarScroll::Down) {
        selected_slot.0 = (selected_slot.0 - 1) % HOTBAR_SLOTS;
    }
}

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedHotbarSlot(0))
            .insert_resource(UITextureMap(HashMap::new()))
            .add_systems(Startup, setup_hotbar_display)
            .add_systems(Update, update_hotbar)
            .add_systems(Update, scroll_hotbar)
            ;
    }
}