use bevy::prelude::Component;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash, Component)]
pub enum Realm {
    #[default]
    Overworld,
    Aether,
    Nether
}