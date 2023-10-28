mod terrain_gen;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_cols;
pub use terrain_gen::Generators;
pub use earth_gen::Earth;
pub use debug_gen::DebugGen;
pub use load_area::LoadArea;
pub use load_cols::*;
// TODO: remove when water is represented in bloc data
pub use earth_gen::WATER_H;