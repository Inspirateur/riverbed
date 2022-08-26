use bevy::ecs::component::Component;

#[derive(Component, PartialEq, Eq, Clone, Copy, Default, Debug, Hash)]
pub enum Realm {
    #[default]
    Earth,
    Abyss,
    Celestial,
}
