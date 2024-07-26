use std::collections::HashMap;

use bevy::{audio::PlaybackMode, prelude::*};
use rand::Rng;
use crate::{agents::{BlockActionType, BlockLootAction, SteppingOn, Velocity}, blocks::{Block, BlockFamily}, items::BlockKind};
const RAND_AMPLITUDE: f32 = 0.3;
// distance between steps (in blocks)
const STEP_DIST: f32 = 3.0;

pub struct BlockSoundPlugin;

impl Plugin for BlockSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, load_block_sounds)
            .add_systems(Update, footsteps)
            .add_systems(Update, breaking)
            ;
    }
}

enum BlockSound {
    Stepping,
    Breaking,
    Harvesting,
}

#[derive(Resource)]
struct BlockSounds {
    stepping: HashMap<BlockKind, Handle<AudioSource>>,
    breaking: HashMap<BlockKind, Handle<AudioSource>>,
    harvesting: HashMap<BlockKind, Handle<AudioSource>>,
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

fn load_block_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO: make it automatically associate sound and block kinds (like we do for block texture)
    let mut stepping_sounds = HashMap::new();
    stepping_sounds.insert(BlockKind::Family(BlockFamily::Soil), asset_server.load("sounds/blocks/footsteps/fwip.ogg"));
    let mut breaking_sounds = HashMap::new();
    breaking_sounds.insert(BlockKind::Family(BlockFamily::Soil), asset_server.load("sounds/blocks/breaking/Soil.ogg"));
    breaking_sounds.insert(BlockKind::Family(BlockFamily::Log), asset_server.load("sounds/blocks/breaking/Log.ogg"));
    breaking_sounds.insert(BlockKind::Family(BlockFamily::Stone), asset_server.load("sounds/blocks/breaking/Stone.ogg"));
    breaking_sounds.insert(BlockKind::Family(BlockFamily::Foliage), asset_server.load("sounds/blocks/breaking/Foliage.ogg"));
    let mut harvesting_sounds = HashMap::new();
    harvesting_sounds.insert(BlockKind::Family(BlockFamily::Stone), asset_server.load("sounds/blocks/harvesting/Stone.ogg"));
    commands.insert_resource(BlockSounds { stepping: stepping_sounds, breaking: breaking_sounds, harvesting: harvesting_sounds });
}

#[derive(Component)]
pub struct FootstepCD(pub f32);

fn footsteps(
    mut commands: Commands, 
    block_sounds: Res<BlockSounds>,
    time: Res<Time>, 
    mut steppers_query: Query<(&Transform, &Velocity, &SteppingOn, &mut FootstepCD)>, 
) {
    for (transform, velocity, stepping_on, mut footstep_cd) in steppers_query.iter_mut() {
        footstep_cd.0 -= velocity.0.length()*time.delta_seconds();
        if footstep_cd.0 > 0. {
            continue;
        }
        let Some(sound) = block_sounds.sound_for(stepping_on.0, BlockSound::Stepping) else {
            continue;
        };
        commands.spawn(SpatialBundle::from_transform(transform.clone())).insert(AudioSourceBundle {
            source: sound.clone(),
            settings: PlaybackSettings { 
                mode: PlaybackMode::Despawn, 
                speed: 1.+((rand::thread_rng().gen::<f32>()-0.5)*RAND_AMPLITUDE),
                ..Default::default()
            }
        });
        footstep_cd.0 = STEP_DIST;
    }
}

#[derive(Component)]
pub struct BlockSoundCD(pub f32);

fn breaking(
    mut commands: Commands, 
    block_sounds: Res<BlockSounds>,
    time: Res<Time>, 
    mut looting_query: Query<(&BlockLootAction, &mut BlockSoundCD)>
) {
    for (looting_action, mut sound_cd) in looting_query.iter_mut() {
        sound_cd.0 -= time.delta_seconds();
        if sound_cd.0 > 0. {
            continue;
        }
        let Some(sound) = block_sounds.sound_for(looting_action.block, match looting_action.action_type {
            BlockActionType::Breaking => BlockSound::Breaking,
            BlockActionType::Harvesting => BlockSound::Harvesting,
        }) else {
            continue;
        };
        commands.spawn(SpatialBundle::from_transform(Transform::from_translation(looting_action.block_pos.into())))
            .insert(AudioSourceBundle {
                source: sound.clone(),
                settings: PlaybackSettings { 
                    mode: PlaybackMode::Despawn, 
                    speed: 1.+((rand::thread_rng().gen::<f32>()-0.5)*RAND_AMPLITUDE),
                    ..Default::default()
                }
            });
        sound_cd.0 = 0.3;
    }
}