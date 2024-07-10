use std::collections::HashMap;
use bevy::{asset::LoadedFolder, color::palettes::css, prelude::*};
use leafwing_input_manager::prelude::*;
use crate::{agents::PlayerControlled, blocks::FaceSpecifier, items::{Hotbar, Item, Stack, HOTBAR_SLOTS}, render::{parse_block_tex_name, parse_item_tex_name, BlockTexState, BlockTextureFolder, ItemTexState, ItemTextureFolder}};
const SLOT_SIZE_PERCENT: f32 = 6.;

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Resource)]
struct UITextureMap(HashMap<Item, Handle<Image>>);

#[derive(Resource)]
pub struct SelectedHotbarSlot(pub usize);

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum HotbarScroll {
    Left, 
    Right,
}

fn load_hotbar_item_textures(
    item_textures: Res<ItemTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut ui_tex_map: ResMut<UITextureMap>,
) {
    let item_folder: &LoadedFolder = loaded_folders.get(&item_textures.0).unwrap();
    for item_handle in item_folder.handles.iter() {
        let filename = item_handle.path().unwrap().path().file_stem().unwrap();
        let Some(item) = parse_item_tex_name(filename) else {
            continue;
        };
        ui_tex_map.0.insert(item, item_handle.clone().try_typed().unwrap());
    }
}

fn load_hotbar_block_textures(
    block_textures: Res<BlockTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut ui_tex_map: ResMut<UITextureMap>,
) {
    let block_folder: &LoadedFolder = loaded_folders.get(&block_textures.0).unwrap();
    let mut priority_map: HashMap<Item, FaceSpecifier> = HashMap::new();
    for block_handle in block_folder.handles.iter() {        
        let filename = block_handle.path().unwrap().path().file_stem().unwrap();
        let Some((block, face_specifier)) = parse_block_tex_name(filename) else {
            continue;
        };
        priority_map.insert(Item::Block(block), face_specifier);
        ui_tex_map.0.insert(Item::Block(block), block_handle.clone().try_typed().unwrap());
    }
}

fn setup_hotbar_display(
    mut commands: Commands, asset_server: Res<AssetServer>
) {
    let input_map = InputMap::default()
        .insert(HotbarScroll::Left, MouseWheelDirection::Up)
        .insert(HotbarScroll::Right, MouseWheelDirection::Down)
        .build()
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

fn display_hotbar(
    mut bg_query: Query<(&mut BackgroundColor, &HotbarSlot)>, 
    mut img_query: Query<(&mut UiImage, &HotbarSlot)>, 
    mut text_query: Query<(&mut Text, &HotbarSlot)>,
    tex_map: Res<UITextureMap>,
    selected_slot: Res<SelectedHotbarSlot>,
    hotbar_query: Query<&Hotbar, (With<PlayerControlled>, Changed<Hotbar>)>,
) {
    // Highlight the selected slot with a darker bg color
    for (mut bg, slot) in bg_query.iter_mut() {
        bg.0 = if slot.0 == selected_slot.0 {
            Color::linear_rgba(0., 0., 0., 0.6)
        } else {
            Color::linear_rgba(0., 0., 0., 0.3)
        };
    }
    let Ok(hotbar) = hotbar_query.get_single() else {
        // The hotbar hasn't changed since last time
        return;
    };
    for (mut img, slot) in img_query.iter_mut() {
        *img = if let Stack::Some(item, _) = hotbar.0.0[slot.0] {
            if let Some(handle) = tex_map.0.get(&item) {
                UiImage::new(handle.clone_weak())
            } else {
                UiImage::solid_color(Color::NONE)
            }
        } else {
            UiImage::solid_color(Color::NONE)
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
    if action_state.pressed(&HotbarScroll::Left) {
        selected_slot.0 = (selected_slot.0 - 1).rem_euclid(HOTBAR_SLOTS);
    } else if action_state.pressed(&HotbarScroll::Right) {
        selected_slot.0 = (selected_slot.0 + 1).rem_euclid(HOTBAR_SLOTS);
    }
}

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(InputManagerPlugin::<HotbarScroll>::default())
            .insert_resource(SelectedHotbarSlot(0))
            .insert_resource(UITextureMap(HashMap::new()))
            .add_systems(Startup, setup_hotbar_display)
            .add_systems(OnEnter(BlockTexState::Finished), load_hotbar_block_textures)
            .add_systems(OnEnter(ItemTexState::Finished), load_hotbar_item_textures)
            .add_systems(Update, display_hotbar)
            .add_systems(Update, scroll_hotbar)
            ;
    }
}