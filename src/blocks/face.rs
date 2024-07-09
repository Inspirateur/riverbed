use bevy::ecs::component::Component;
use strum_macros::EnumIter;
const UP_SPECIFIER: [FaceSpecifier; 2] = [FaceSpecifier::Specific(Face::Up), FaceSpecifier::All];
const DOWN_SPECIFIER: [FaceSpecifier; 3] = [FaceSpecifier::Specific(Face::Down), FaceSpecifier::Specific(Face::Up), FaceSpecifier::All];
const LEFT_SPECIFIER: [FaceSpecifier; 3] = [FaceSpecifier::Specific(Face::Left), FaceSpecifier::Side, FaceSpecifier::All];
const RIGHT_SPECIFIER: [FaceSpecifier; 3] = [FaceSpecifier::Specific(Face::Right), FaceSpecifier::Side, FaceSpecifier::All];
const FRONT_SPECIFIER: [FaceSpecifier; 3] = [FaceSpecifier::Specific(Face::Front), FaceSpecifier::Side, FaceSpecifier::All];
const BACK_SPECIFIER: [FaceSpecifier; 3] = [FaceSpecifier::Specific(Face::Back), FaceSpecifier::Side, FaceSpecifier::All];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FaceSpecifier {
    Specific(Face),
    Side,
    All
}

#[derive(Component, EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Face {
    Left,
    Down,
    Back,
    Right,
    Up,
    Front,
}

impl Face {
    pub fn n(&self) -> [i32; 3] {
        match self {
            Self::Left => [-1, 0, 0],
            Self::Down => [0, -1, 0],
            Self::Back => [0, 0, -1], 
            Self::Right => [1, 0, 0],
            Self::Up => [0, 1, 0],
            Self::Front => [0, 0, 1],
        }
    }

    pub fn specifiers(&self) -> &[FaceSpecifier] {
        match self {
            Self::Left => &LEFT_SPECIFIER,
            Self::Down => &DOWN_SPECIFIER,
            Self::Back => &BACK_SPECIFIER, 
            Self::Right => &RIGHT_SPECIFIER,
            Self::Up => &UP_SPECIFIER,
            Self::Front => &FRONT_SPECIFIER, 
        }
    }
}

// must match RIGHT_HANDED_Y_UP_CONFIG.faces from block-mesh-rs
impl From<u8> for Face {
    fn from(value: u8) -> Self {
        assert!(value < 6);
        match value {
            0 => Self::Left,
            1 => Self::Down,
            2 => Self::Back,
            3 => Self::Right,
            4 => Self::Up,
            5 => Self::Front,
            _ => unreachable!(),
        }
    }
}

impl From<usize> for Face {
    fn from(value: usize) -> Self {
        (value as u8).into()
    }
}
