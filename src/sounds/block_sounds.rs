use bevy::{audio::PlaybackMode, prelude::*, utils::HashMap};
use rand::Rng;
use crate::{agents::{BreakingAction, SteppingOn, Velocity}, blocks::{Block, BlockFamily}, items::BlockKind};
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

enum BlockAction {
    Stepping,
    Breaking,
}

#[derive(Resource)]
struct BlockSounds {
    stepping: HashMap<BlockKind, Handle<AudioSource>>,
    breaking: HashMap<BlockKind, Handle<AudioSource>>,
}

impl BlockSounds {
    pub fn sound_for(&self, block: Block, action: BlockAction) -> Option<&Handle<AudioSource>> {
        let map = match action {
            BlockAction::Stepping => &self.stepping,
            BlockAction::Breaking => &self.breaking,
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
    let mut stepping_sounds = HashMap::new();
    stepping_sounds.insert(BlockKind::Family(BlockFamily::Soil), asset_server.load("sounds/blocks/footsteps/fwip.ogg"));
    let mut breaking_sounds = HashMap::new();
    breaking_sounds.insert(BlockKind::Family(BlockFamily::Soil), asset_server.load("sounds/blocks/breaking/krr.ogg"));
    commands.insert_resource(BlockSounds { stepping: stepping_sounds, breaking: breaking_sounds });
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
        let Some(sound) = block_sounds.sound_for(stepping_on.0, BlockAction::Stepping) else {
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
    mut breaking_query: Query<(&BreakingAction, &mut BlockSoundCD)>
) {
    for (breaking_action, mut sound_cd) in breaking_query.iter_mut() {
        sound_cd.0 -= time.delta_seconds();
        if sound_cd.0 > 0. {
            continue;
        }
        let Some(sound) = block_sounds.sound_for(breaking_action.block, BlockAction::Breaking) else {
            continue;
        };
        commands.spawn(SpatialBundle::from_transform(Transform::from_translation(breaking_action.block_pos.into())))
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