use bevy::{audio::PlaybackMode, prelude::*};
use rand::Rng;
use crate::agents::{BlockActionType, BlockLootAction, SteppingOn, Velocity};
use super::block_sound_load::{BlockSound, BlockSoundLoadPlugin, BlockSounds};
const RAND_AMPLITUDE: f32 = 0.3;
// distance between steps (in blocks)
const STEP_DIST: f32 = 2.5;

pub struct BlockSoundPlugin;

impl Plugin for BlockSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugins(BlockSoundLoadPlugin)
            .add_systems(Update, footsteps)
            .add_systems(Update, breaking)
            ;
    }
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