use std::{collections::HashMap, str::FromStr};
use bevy::{asset::LoadedFolder, prelude::*};
use crate::{Block, items::BlockKind};
// TODO: find a way to make this less verbose ?
pub struct BlockSoundLoadPlugin;

impl Plugin for BlockSoundLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<BreakSoundsState>()
            .init_state::<HarvestSoundsState>()
            .init_state::<StepSoundsState>()
            .insert_resource(BlockSounds::default())
            .add_systems(OnEnter(BreakSoundsState::Setup), load_block_breaking_sounds)
            .add_systems(Update, check_break_sounds.run_if(in_state(BreakSoundsState::Setup)))
            .add_systems(OnEnter(HarvestSoundsState::Setup), load_block_harvesting_sounds)
            .add_systems(Update, check_harvest_sounds.run_if(in_state(HarvestSoundsState::Setup)))
            .add_systems(OnEnter(StepSoundsState::Setup), load_block_stepping_sounds)
            .add_systems(Update, check_step_sounds.run_if(in_state(StepSoundsState::Setup)))
            .add_systems(OnEnter(BreakSoundsState::Finished), load_block_break_sounds)
            .add_systems(OnEnter(HarvestSoundsState::Finished), load_block_harvest_sounds)
            .add_systems(OnEnter(StepSoundsState::Finished), load_block_step_sounds)
            ;
    }
}

pub enum BlockSound {
    Stepping,
    Breaking,
    Harvesting,
}

#[derive(Default, Resource)]
pub struct BlockSounds {
    pub breaking: HashMap<BlockKind, Handle<AudioSource>>,
    pub harvesting: HashMap<BlockKind, Handle<AudioSource>>,
    pub stepping: HashMap<BlockKind, Handle<AudioSource>>,
}

impl BlockSounds {
    pub fn sound_for(&self, block: Block, sound_type: BlockSound) -> Option<&Handle<AudioSource>> {
        let map = match sound_type {
            BlockSound::Stepping => &self.stepping,
            BlockSound::Breaking => &self.breaking,
            BlockSound::Harvesting => &self.harvesting,
        };
        if let Some(sound) = map.get(&BlockKind::Block(block)) {
            return Some(sound);
        }
        for family in block.families() {
            if let Some(sound) = map.get(&BlockKind::Family(family)) {
                return Some(sound);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum BreakSoundsState {
    #[default]
    Setup,
    Finished
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum HarvestSoundsState {
    #[default]
    Setup,
    Finished
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum StepSoundsState {
    #[default]
    Setup,
    Finished
}

#[derive(Resource, Default)]
pub struct BreakSoundFolder(pub Handle<LoadedFolder>);

#[derive(Resource, Default)]
pub struct HarvestSoundFolder(pub Handle<LoadedFolder>);

#[derive(Resource, Default)]
pub struct StepSoundFolder(pub Handle<LoadedFolder>);


fn load_block_breaking_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(BreakSoundFolder(asset_server.load_folder("sounds/blocks/breaking")));
}

fn load_block_harvesting_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(HarvestSoundFolder(asset_server.load_folder("sounds/blocks/harvesting")));
}

fn load_block_stepping_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(StepSoundFolder(asset_server.load_folder("sounds/blocks/footsteps")));
}

fn check_break_sounds(
    mut next_state: ResMut<NextState<BreakSoundsState>>,
    texture_folder: ResMut<BreakSoundFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(BreakSoundsState::Finished);
        }
    }
}

fn check_harvest_sounds(
    mut next_state: ResMut<NextState<HarvestSoundsState>>,
    texture_folder: ResMut<HarvestSoundFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(HarvestSoundsState::Finished);
        }
    }
}

fn check_step_sounds(
    mut next_state: ResMut<NextState<StepSoundsState>>,
    texture_folder: ResMut<StepSoundFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(StepSoundsState::Finished);
        }
    }
}

fn load_block_break_sounds(
    break_sound_folder: Res<BreakSoundFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut block_sounds: ResMut<BlockSounds>,
) {
    let loaded_folder: &LoadedFolder = loaded_folders.get(&break_sound_folder.0).unwrap();
    for handle in loaded_folder.handles.iter() {
        let Some(filename) = handle.path().unwrap().path().file_stem().unwrap().to_str() else {
            continue;
        };
        block_sounds.breaking.insert(BlockKind::from_str(filename).unwrap(), handle.clone().typed());
    }
}

fn load_block_harvest_sounds(
    harvest_sound_folder: Res<HarvestSoundFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut block_sounds: ResMut<BlockSounds>,
) {
    let loaded_folder: &LoadedFolder = loaded_folders.get(&harvest_sound_folder.0).unwrap();
    for handle in loaded_folder.handles.iter() {
        let Some(filename) = handle.path().unwrap().path().file_stem().unwrap().to_str() else {
            continue;
        };
        block_sounds.harvesting.insert(BlockKind::from_str(filename).unwrap(), handle.clone().typed());
    }
}

fn load_block_step_sounds(
    step_sound_folder: Res<StepSoundFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut block_sounds: ResMut<BlockSounds>,
) {
    let loaded_folder: &LoadedFolder = loaded_folders.get(&step_sound_folder.0).unwrap();
    for handle in loaded_folder.handles.iter() {
        let Some(filename) = handle.path().unwrap().path().file_stem().unwrap().to_str() else {
            continue;
        };
        block_sounds.stepping.insert(BlockKind::from_str(filename).unwrap(), handle.clone().typed());
    }
}
