mod block_sounds;
mod ui_sounds;
mod effect_sounds;
mod block_sound_load;
use bevy::app::Plugin;
use block_sounds::BlockSoundPlugin;
use effect_sounds::EffectSoundPlugin;
use ui_sounds::UiSoundPlugin;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugins(UiSoundPlugin)
            .add_plugins(BlockSoundPlugin)
            .add_plugins(EffectSoundPlugin)
            ;
    }
}