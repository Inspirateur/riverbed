mod block_breaking;
use bevy::app::Plugin;
use block_breaking::BlockBreakingEffectPlugin;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugins(BlockBreakingEffectPlugin)
            ;
    }
}