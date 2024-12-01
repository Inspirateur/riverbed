use crate::ui::{Dragging, SelectedHotbarSlot};
use bevy::{audio::PlaybackMode, prelude::*};

pub struct UiSoundPlugin;

impl Plugin for UiSoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_ui_sounds)
            .add_systems(Update, hotbar_slot_change)
            .add_systems(Update, on_item_slot_click);
    }
}

#[derive(Resource)]
struct UiSounds {
    hotbar_slot: Handle<AudioSource>,
    item_clicked: Handle<AudioSource>,
}

fn setup_ui_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiSounds {
        hotbar_slot: asset_server.load("sounds/ui/k.ogg"),
        item_clicked: asset_server.load("sounds/ui/kic.ogg"),
    });
}

fn hotbar_slot_change(
    mut commands: Commands,
    ui_sounds: Res<UiSounds>,
    selected_slot: Res<SelectedHotbarSlot>,
) {
    if selected_slot.is_changed() {
        commands.spawn((
            AudioPlayer::<AudioSource>(ui_sounds.hotbar_slot.clone_weak()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..Default::default()
            },
        ));
    }
}

fn on_item_slot_click(
    mut commands: Commands,
    ui_sounds: Res<UiSounds>,
    dragged_item: Res<Dragging>,
) {
    if dragged_item.is_changed() {
        commands.spawn((
            AudioPlayer::<AudioSource>(ui_sounds.item_clicked.clone_weak()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..Default::default()
            },
        ));
    }
}

