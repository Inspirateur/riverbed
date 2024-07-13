use bevy::{audio::PlaybackMode, prelude::*};
use crate::ui::SelectedHotbarSlot;

pub struct UiSoundPlugin;

impl Plugin for UiSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(Startup, setup_ui_sounds)
            .add_systems(Update, hotbar_slot_change)
            ;
    }
}

#[derive(Resource)]
struct UiSounds {
    hotbar_slot: Handle<AudioSource>,
}

fn setup_ui_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiSounds {
        hotbar_slot: asset_server.load("sounds/ui/t.ogg"),
    });
}

fn hotbar_slot_change(
    mut commands: Commands, 
    ui_sounds: Res<UiSounds>,
    selected_slot: Res<SelectedHotbarSlot>, 
) {
    if selected_slot.is_changed() {
        commands.spawn(AudioSourceBundle {
            source: ui_sounds.hotbar_slot.clone_weak(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..Default::default()
            }
        });
    }
}
