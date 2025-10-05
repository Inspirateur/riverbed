use crate::{
    agents::{BlockPlaced, Furnace},
    items::LitFurnace,
};
use bevy::{
    audio::{PlaybackMode, SpatialScale},
    prelude::*,
};
use rand::Rng;
const RAND_AMPLITUDE: f32 = 0.3;

pub struct EffectSoundPlugin;

impl Plugin for EffectSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_effect_sounds)
            .add_systems(Update, setup_furnace_cd)
            .add_systems(Update, furnace_sounds)
            .add_observer(on_block_placed);
    }
}

#[derive(Resource)]
pub struct EffectSounds {
    item_get: Handle<AudioSource>,
    fire_crackle: Handle<AudioSource>,
    flame: Handle<AudioSource>,
    block_placed: Handle<AudioSource>,
}

fn setup_effect_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(EffectSounds {
        item_get: asset_server.load("sounds/effects/pop.ogg"),
        fire_crackle: asset_server.load("sounds/effects/tt.ogg"),
        flame: asset_server.load("sounds/effects/flame.ogg"),
        block_placed: asset_server.load("sounds/effects/p.ogg"),
    });
}

#[derive(EntityEvent)]
pub struct ItemGet {
    pub entity: Entity,
}

pub fn on_item_get(_: On<ItemGet>, mut commands: Commands, effect_sounds: Res<EffectSounds>) {
    commands.spawn((
        AudioPlayer::<AudioSource>(effect_sounds.item_get.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..Default::default()
        },
    ));
}

pub fn on_block_placed(
    block_placed: On<BlockPlaced>,
    mut commands: Commands,
    effect_sounds: Res<EffectSounds>,
) {
    commands
        .spawn((
            Transform::from_translation(block_placed.event().0.into()),
            Visibility::default(),
        ))
        .insert((
            AudioPlayer::<AudioSource>(effect_sounds.block_placed.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                spatial: true,
                spatial_scale: Some(SpatialScale::new(0.2)),
                ..Default::default()
            },
        ));
}

#[derive(Component)]
struct FireCrackleCD(pub f32);

#[derive(Component)]
struct FlameCD(pub f32);

fn setup_furnace_cd(
    mut commands: Commands,
    furnace_query: Query<Entity, (With<LitFurnace>, Without<FlameCD>)>,
) {
    for furnace in furnace_query.iter() {
        commands
            .entity(furnace)
            .insert((FireCrackleCD(0.), FlameCD(0.)));
    }
}

fn furnace_sounds(
    mut commands: Commands,
    effect_sounds: Res<EffectSounds>,
    time: Res<Time>,
    mut furnace_query: Query<(&Furnace, &mut FireCrackleCD, &mut FlameCD), With<LitFurnace>>,
) {
    for (furnace, mut fire_crackle_cd, mut flame_cd) in furnace_query.iter_mut() {
        fire_crackle_cd.0 -= time.delta_secs();
        if fire_crackle_cd.0 <= 0. {
            commands
                .spawn((
                    Transform::from_translation(furnace.block_pos.into()),
                    Visibility::default(),
                ))
                .insert((
                    AudioPlayer::<AudioSource>(effect_sounds.fire_crackle.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        spatial: true,
                        spatial_scale: Some(SpatialScale::new(0.5)),
                        speed: 1. + ((rand::rng().random::<f32>() - 0.5) * RAND_AMPLITUDE),
                        ..Default::default()
                    },
                ));
            fire_crackle_cd.0 = 0.2 + rand::rng().random::<f32>();
        }
        flame_cd.0 -= time.delta_secs();
        if flame_cd.0 <= 0. {
            commands
                .spawn((
                    Transform::from_translation(furnace.block_pos.into()),
                    Visibility::default(),
                ))
                .insert((
                    AudioPlayer::<AudioSource>(effect_sounds.flame.clone()),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        spatial: true,
                        spatial_scale: Some(SpatialScale::new(0.2)),
                        speed: 1. + ((rand::rng().random::<f32>() - 0.5) * RAND_AMPLITUDE),
                        ..Default::default()
                    },
                ));
            flame_cd.0 = 2.;
        }
    }
}

