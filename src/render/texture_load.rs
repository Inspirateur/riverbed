use std::ffi::OsStr;
use bevy::{asset::LoadedFolder, prelude::*};
use crate::{Block, block::{Face, FaceSpecifier}, asset_processing::from_filename};

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub struct TextureLoadPlugin;

impl Plugin for TextureLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<BlockTexState>()
            .init_state::<ItemTexState>()
            .add_systems(OnEnter(BlockTexState::Setup), load_block_textures)
            .add_systems(Update, check_block_textures.run_if(in_state(BlockTexState::Setup)))
            .add_systems(OnEnter(ItemTexState::Setup), load_item_textures)
            .add_systems(Update, check_item_textures.run_if(in_state(ItemTexState::Setup)))
            ;
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum BlockTexState {
    #[default]
    Setup,
    Loaded,
    Mapped
}

#[derive(Resource, Default)]
pub struct BlockTextureFolder(pub Handle<LoadedFolder>);

fn load_block_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(BlockTextureFolder(asset_server.load_folder("textures/blocks")));
}

fn check_block_textures(
    mut next_state: ResMut<NextState<BlockTexState>>,
    texture_folder: ResMut<BlockTextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(BlockTexState::Loaded);
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ItemTexState {
    #[default]
    Setup,
    Loaded,
    Mapped
}

#[derive(Resource, Default)]
pub struct ItemTextureFolder(pub Handle<LoadedFolder>);


pub fn parse_block_tex_name(filename: &OsStr) -> Option<(Block, FaceSpecifier)> {
    let filename = filename.to_str()?.trim_end_matches(DIGITS);
    let (block, face) = match filename.rsplit_once("_") {
        Some((block, "side")) => (block, FaceSpecifier::Side),
        Some((block, "bottom")) => (block, FaceSpecifier::Specific(Face::Down)),
        Some((block, "top")) => (block, FaceSpecifier::Specific(Face::Up)),
        Some((block, "front")) => (block, FaceSpecifier::Specific(Face::Front)),
        Some((block, "back")) => (block, FaceSpecifier::Specific(Face::Back)),
        _ => (filename, FaceSpecifier::All),
    };
    Some((from_filename(block)?, face))
}

fn load_item_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(ItemTextureFolder(asset_server.load_folder("textures/item")));
}

fn check_item_textures(
    mut next_state: ResMut<NextState<ItemTexState>>,
    texture_folder: ResMut<ItemTextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(ItemTexState::Loaded);
        }
    }
}
