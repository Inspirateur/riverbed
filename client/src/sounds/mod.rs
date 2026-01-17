mod block_sound_load;
mod block_sounds;
mod effect_sounds;
mod ui_sounds;
use bevy::app::Plugin;
use block_sounds::BlockSoundPlugin;
pub use block_sounds::{BlockSoundCD, FootstepCD};
use effect_sounds::EffectSoundPlugin;
pub use effect_sounds::{on_item_get, ItemGet};
use ui_sounds::UiSoundPlugin;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(UiSoundPlugin)
            .add_plugins(BlockSoundPlugin)
            .add_plugins(EffectSoundPlugin);
    }
}
