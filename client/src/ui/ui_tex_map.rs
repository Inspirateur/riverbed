use std::collections::HashMap;
use bevy::{asset::LoadedFolder, color::palettes::css, image::TRANSPARENT_IMAGE_HANDLE, prelude::*};
use itertools::Itertools;
use crate::{asset_processing::from_filename, block::{Face, FaceSpecifier}, items::{Item, Stack}, render::{parse_block_tex_name, BlockTexState, BlockTextureFolder, ItemTexState, ItemTextureFolder}, Block};
pub const SLOT_SIZE_PERCENT: f32 = 4.;

pub struct UiTexMapPlugin;

impl Plugin for UiTexMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UiTextureMapState::default())
            .insert_resource(UiTextureMap(HashMap::new()))
            .add_systems(OnEnter(BlockTexState::Loaded), load_ui_block_textures)
            .add_systems(OnEnter(ItemTexState::Loaded), load_ui_item_textures)
            ;
    }
}

#[derive(PartialEq, Eq)]
pub enum UiSlotKind {
    Default,
    Disabled,
    NoBg
}

#[derive(Default, Resource)]
struct UiTextureMapState {
    pub item_loaded: bool,
    pub block_loaded: bool
}

#[derive(Resource)]
pub struct UiTextureMap(HashMap<Item, Handle<Image>>);

impl UiTextureMap {
    pub fn get_texture(&self, stack: &Stack) -> Handle<Image> {
        match stack {
			Stack::Some(item, _) => self.0.get(item).cloned(),
			Stack::None => None,
		}.unwrap_or(TRANSPARENT_IMAGE_HANDLE)
    }

    pub fn make_empty_item_slot(node: &mut ChildSpawnerCommands, kind: UiSlotKind) {
        let empty_map = UiTextureMap(HashMap::default());
        empty_map.make_item_slot(node, &Stack::None, kind);
    }

	pub fn make_item_slot(&self, node: &mut ChildSpawnerCommands, stack: &Stack, kind: UiSlotKind) {
		let alpha = if kind == UiSlotKind::Disabled { 0.4 } else { 1. };
		node.spawn((
            ImageNode {
                image: self.get_texture(stack),
                color: match stack {
                    Stack::Some(Item::Block(block), _) if block.is_foliage() => Color::linear_rgba(0.3, 1.0, 0.1, alpha),
                    _ => Color::linear_rgba(1., 1., 1., alpha)
                },
                ..Default::default()
            },
            Node {
                width: Val::Vw(SLOT_SIZE_PERCENT),
                aspect_ratio: Some(1.),
                margin: UiRect::all(Val::Percent(0.2)), 
                ..Default::default()
            },
            BackgroundColor(if kind == UiSlotKind::Disabled || kind == UiSlotKind::NoBg {
				Color::NONE
            } else {				
                Color::linear_rgba(0., 0., 0., 0.7)
            })
        ));
        let qty = stack.quantity();
		node.spawn((
            Text::new(if qty > 1 { qty.to_string() } else { String::new() }),
            TextColor(if kind == UiSlotKind::Disabled {
                Color::Srgba(css::LIGHT_GRAY)
            } else {
                Color::Srgba(css::WHITE)
            }),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                ..Default::default()

            }
        ));
	}
}

fn load_ui_item_textures(
    item_textures: Res<ItemTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut ui_tex_map: ResMut<UiTextureMap>,
    mut map_state: ResMut<UiTextureMapState>,
    mut next_state: ResMut<NextState<ItemTexState>>
) {
    let item_folder: &LoadedFolder = loaded_folders.get(&item_textures.0).unwrap();
    for item_handle in item_folder.handles.iter() {
        let Some(filename) = item_handle.path().unwrap().path().file_stem().unwrap().to_str() else {
            continue;
        };
        let Some(item) = from_filename(filename) else {
            continue;
        };
        ui_tex_map.0.insert(item, item_handle.clone().try_typed().unwrap());
    }
    map_state.item_loaded = true;
    if map_state.item_loaded && map_state.block_loaded {
        next_state.set(ItemTexState::Mapped);
    }
}

pub trait UiTextureMapTrait {
    fn get_texture(&self, block: Block, face: Face) -> Option<Handle<Image>>;
}

impl UiTextureMapTrait for HashMap<(Block, FaceSpecifier), Handle<Image>> {
	fn get_texture(&self, block: Block, face: Face) -> Option<Handle<Image>> {
		for specifier in face.specifiers() {
			if let Some(i) = self.get(&(block, *specifier)) {
				return Some(i.clone())
			}
		}
		None
	}
}

fn load_ui_block_textures(
    block_textures: Res<BlockTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut ui_tex_map: ResMut<UiTextureMap>,
    mut map_state: ResMut<UiTextureMapState>,
    mut next_state: ResMut<NextState<ItemTexState>>
) {
    let block_folder: &LoadedFolder = loaded_folders.get(&block_textures.0).unwrap();
    let mut tex_map: HashMap<(Block, FaceSpecifier), Handle<Image>> = HashMap::new();
    for block_handle in block_folder.handles.iter() {        
        let filename = block_handle.path().unwrap().path().file_stem().unwrap();
        let Some((block, face_specifier)) = parse_block_tex_name(filename) else {
            continue;
        };
		tex_map.insert((block, face_specifier), block_handle.clone().try_typed().unwrap());
    }
	for block in tex_map.keys().map(|(block, _)| block).unique() {
		let Some(front_tex) = tex_map.get_texture(*block, Face::Front) else {
			continue;
		};
		// TODO: combine 3 block faces into a perspective view of the block and add it to textures
		ui_tex_map.0.insert(Item::Block(*block), front_tex);
	}
    map_state.block_loaded = true;
    if map_state.item_loaded && map_state.block_loaded {
        next_state.set(ItemTexState::Mapped);
    }
}
