use bevy::ecs::component::Component;

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum Realm {
    Earth,
    Abyss,
    Celestial,
}
