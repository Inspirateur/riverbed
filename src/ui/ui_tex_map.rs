use std::collections::HashMap;
use bevy::{asset::LoadedFolder, color::palettes::css, prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE};
use itertools::Itertools;
use crate::{asset_processing::from_filename, block::{Face, FaceSpecifier}, items::{Item, Stack}, render::{parse_block_tex_name, BlockTexState, BlockTextureFolder, ItemTexState, ItemTextureFolder}, Block};
pub const SLOT_SIZE_PERCENT: f32 = 4.;

pub struct UiTexMapPlugin;

impl Plugin for UiTexMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UiTextureMap(HashMap::new()))
            .add_systems(OnEnter(BlockTexState::Loaded), load_ui_block_textures)
            .add_systems(OnEnter(ItemTexState::Loaded), load_ui_item_textures)
            ;
    }
}

#[derive(Resource)]
pub struct UiTextureMap(pub HashMap<Item, Handle<Image>>);

impl UiTextureMap {
    pub fn get_with_alpha(&self, stack: &Stack, alpha: f32) -> UiImage {
        UiImage::new(match stack {
			Stack::Some(item, _) => self.0.get(item).cloned(),
			Stack::None => None,
		}.unwrap_or(TRANSPARENT_IMAGE_HANDLE)).with_color({
            match stack {
                Stack::Some(Item::Block(block), _) if block.is_foliage() => Color::linear_rgba(0.3, 1.0, 0.1, alpha),
                _ => Color::linear_rgba(1., 1., 1., alpha)
            }
        })
    }

    pub fn make_empty_item_slot(node: &mut ChildBuilder, disabled: bool) {
        let empty_map = UiTextureMap(HashMap::default());
        empty_map.make_item_slot(node, &Stack::None, disabled);
    }

	pub fn make_item_slot(&self, node: &mut ChildBuilder, stack: &Stack, disabled: bool) {
		let alpha = if disabled { 0.4 } else { 1. };
		node.spawn(ImageBundle {
            style: Style {
                width: Val::Vw(SLOT_SIZE_PERCENT),
                aspect_ratio: Some(1.),
                margin: UiRect::all(Val::Percent(0.2)), 
                ..Default::default()
            },
            image: self.get_with_alpha(stack, alpha),
            background_color: BackgroundColor(if disabled {
				Color::NONE
            } else {				
                Color::linear_rgba(0., 0., 0., 0.7)
            }),
            ..Default::default()
        });
        let qty = stack.quantity();
		node.spawn(TextBundle {
            text: Text::from_section(if qty > 1 { qty.to_string() } else { String::new() }, TextStyle { 
                color: if disabled {
					Color::Srgba(css::LIGHT_GRAY)
                } else {
					Color::Srgba(css::WHITE)
                }, ..Default::default() }),
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                ..Default::default()
            },
            ..Default::default() 
        });
	}
}

fn load_ui_item_textures(
    item_textures: Res<ItemTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut ui_tex_map: ResMut<UiTextureMap>,
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
}
