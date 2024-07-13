use bevy::{audio::PlaybackMode, prelude::*};

pub struct EffectSoundPlugin;

impl Plugin for EffectSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, setup_effect_sounds)
            ;
    }
}

#[derive(Resource)]
pub struct EffectSounds {
    item_get: Handle<AudioSource>,
}

fn setup_effect_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(EffectSounds {
        item_get: asset_server.load("sounds/effects/pop.ogg"),
    });
}

#[derive(Event)]
pub struct ItemGet;

pub fn on_item_get(_: Trigger<ItemGet>, mut commands: Commands, effect_sounds: Res<EffectSounds>) {
    commands.spawn(AudioSourceBundle {
        source: effect_sounds.item_get.clone_weak(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..Default::default()
        }
    });
}