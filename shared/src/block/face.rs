use bevy::ecs::component::Component;
use strum_macros::EnumIter;
const UP_SPECIFIER: [FaceSpecifier; 2] = [FaceSpecifier::Specific(Face::Up), FaceSpecifier::All];
const DOWN_SPECIFIER: [FaceSpecifier; 3] = [
    FaceSpecifier::Specific(Face::Down),
    FaceSpecifier::Specific(Face::Up),
    FaceSpecifier::All,
];
const LEFT_SPECIFIER: [FaceSpecifier; 3] = [
    FaceSpecifier::Specific(Face::Left),
    FaceSpecifier::Side,
    FaceSpecifier::All,
];
const RIGHT_SPECIFIER: [FaceSpecifier; 3] = [
    FaceSpecifier::Specific(Face::Right),
    FaceSpecifier::Side,
    FaceSpecifier::All,
];
const FRONT_SPECIFIER: [FaceSpecifier; 3] = [
    FaceSpecifier::Specific(Face::Front),
    FaceSpecifier::Side,
    FaceSpecifier::All,
];
const BACK_SPECIFIER: [FaceSpecifier; 3] = [
    FaceSpecifier::Specific(Face::Back),
    FaceSpecifier::Side,
    FaceSpecifier::All,
];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FaceSpecifier {
    Specific(Face),
    Side,
    All,
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

    pub fn quad_to_block(&self) -> [i32; 3] {
        match self {
            Self::Left => [-1, 0, 0],
            Self::Down => [0, -1, 0],
            Self::Back => [0, 0, -1],
            Self::Right => [0, 0, 0],
            Self::Up => [0, 0, 0],
            Self::Front => [-1, 0, 0],
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

    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Down => Self::Up,
            Self::Back => Self::Front,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Front => Self::Back,
        }
    }
}

// must match RIGHT_HANDED_Y_UP_CONFIG.faces from block-mesh-rs
impl From<u8> for Face {
    fn from(value: u8) -> Self {
        assert!(value < 6);
        match value {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Right,
            3 => Self::Left,
            4 => Self::Front,
            5 => Self::Back,
            _ => unreachable!(),
        }
    }
}

impl From<usize> for Face {
    fn from(value: usize) -> Self {
        (value as u8).into()
    }
}

//
// Note: This whole section will become unnecessary when we have instancing,
// because it is used to convert a quad to 4 vertices which we won't need to do

fn packed_xyz(x: u32, y: u32, z: u32) -> u32 {
    (z << 12) | (y << 6) | x
}

fn vertex_info(xyz: u32, u: u32, v: u32) -> u32 {
    (v << 24) | (u << 18) | xyz
}

impl Face {
    pub fn vertices_packed(&self, xyz: u32, w: u32, h: u32, lod: u32) -> [u32; 4] {
        let xyz = xyz * lod;
        let w_ = w * lod;
        let h_ = h * lod;
        match self {
            Face::Left => [
                vertex_info(xyz, h, w),
                vertex_info(xyz + packed_xyz(0, 0, h_), 0, w),
                vertex_info(xyz + packed_xyz(0, w_, 0), h, 0),
                vertex_info(xyz + packed_xyz(0, w_, h_), 0, 0),
            ],
            Face::Down => [
                vertex_info(xyz - packed_xyz(w_, 0, 0) + packed_xyz(0, 0, h_), w, h),
                vertex_info(xyz - packed_xyz(w_, 0, 0), w, 0),
                vertex_info(xyz + packed_xyz(0, 0, h_), 0, h),
                vertex_info(xyz, 0, 0),
            ],
            Face::Back => [
                vertex_info(xyz, w, h),
                vertex_info(xyz + packed_xyz(0, h_, 0), w, 0),
                vertex_info(xyz + packed_xyz(w_, 0, 0), 0, h),
                vertex_info(xyz + packed_xyz(w_, h_, 0), 0, 0),
            ],
            Face::Right => [
                vertex_info(xyz, 0, 0),
                vertex_info(xyz + packed_xyz(0, 0, h_), h, 0),
                vertex_info(xyz - packed_xyz(0, w_, 0), 0, w),
                vertex_info(xyz + packed_xyz(0, 0, h_) - packed_xyz(0, w_, 0), h, w),
            ],
            Face::Up => [
                vertex_info(xyz + packed_xyz(w_, 0, h_), w, h),
                vertex_info(xyz + packed_xyz(w_, 0, 0), w, 0),
                vertex_info(xyz + packed_xyz(0, 0, h_), 0, h),
                vertex_info(xyz, 0, 0),
            ],
            Face::Front => [
                vertex_info(xyz - packed_xyz(w_, 0, 0) + packed_xyz(0, h_, 0), 0, 0),
                vertex_info(xyz - packed_xyz(w_, 0, 0), 0, h),
                vertex_info(xyz + packed_xyz(0, h_, 0), w, 0),
                vertex_info(xyz, w, h),
            ],
        }
    }
}
