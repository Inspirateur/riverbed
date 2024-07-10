use std::{ffi::OsStr, str::FromStr};
use itertools::Itertools;
use bevy::{asset::LoadedFolder, prelude::*};
use crate::{blocks::{Block, Face, FaceSpecifier}, items::Item};

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
    Finished
}

#[derive(Resource, Default)]
pub struct BlockTextureFolder(pub Handle<LoadedFolder>);

fn load_block_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(BlockTextureFolder(asset_server.load_folder("PixelPerfection/textures/blocks")));
}

fn check_block_textures(
    mut next_state: ResMut<NextState<BlockTexState>>,
    texture_folder: ResMut<BlockTextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(BlockTexState::Finished);
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ItemTexState {
    #[default]
    Setup,
    Finished
}

#[derive(Resource, Default)]
pub struct ItemTextureFolder(pub Handle<LoadedFolder>);

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn parse_block_name(blockname: &str) -> Option<Block> {
    Block::from_str(&blockname.split("_").map(capitalize).join("")).ok()
}

pub fn parse_block_tex_name(filename: &OsStr) -> Option<(Block, FaceSpecifier)> {
    let filename = filename.to_str()?;
    let Some((block, face)) = filename.rsplit_once("_") else {
        return Some((parse_block_name(filename)?, FaceSpecifier::All));
    };
    match face {
        "side" => Some((parse_block_name(block)?, FaceSpecifier::Side)),
        "bottom" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Down))),
        "top" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Up))),
        "front" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Front))),
        "side1" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Left))),
        "side2" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Right))),
        _ => Some((parse_block_name(filename)?, FaceSpecifier::All))
    }
}

fn load_item_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(ItemTextureFolder(asset_server.load_folder("PixelPerfection/textures/item")));
}

fn check_item_textures(
    mut next_state: ResMut<NextState<ItemTexState>>,
    texture_folder: ResMut<ItemTextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(ItemTexState::Finished);
        }
    }
}

pub fn parse_item_tex_name(filename: &OsStr) -> Option<Item> {
    None
}
