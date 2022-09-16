use bevy::ecs::component::Component;
use strum_macros::{EnumCount, EnumIter};

#[derive(Component, PartialEq, Eq, Clone, Copy, Default, Debug, Hash, EnumCount, EnumIter)]
pub enum Realm {
    #[default]
    Earth,
    Abyss,
    Celestial,
}
