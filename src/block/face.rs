use binary_greedy_meshing::Face;
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

pub trait FaceSpecifierTrait {
    fn specifiers(&self) -> &[FaceSpecifier];
}

impl FaceSpecifierTrait for Face {
    fn specifiers(&self) -> &[FaceSpecifier] {
        match self {
            Face::Up => &UP_SPECIFIER,
            Face::Down => &DOWN_SPECIFIER,
            Face::Left => &LEFT_SPECIFIER,
            Face::Right => &RIGHT_SPECIFIER,
            Face::Front => &FRONT_SPECIFIER,
            Face::Back => &BACK_SPECIFIER,
        }
    }
}