mod terrain_gen;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_cols;
pub use terrain_gen::Generators;
pub use earth_gen::Earth;
pub use debug_gen::DebugGen;
pub use load_area::LoadArea;
pub use load_cols::{LoadedCols, ColUnloadEvent};
// TODO: remove when water is represented in bloc data
pub use earth_gen::WATER_H;
use bevy::prelude::{Plugin, Update};
use self::{load_area::{update_load_area, load_order}, load_cols::pull_orders};

pub struct GenPlugin;

impl Plugin for GenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.insert_resource(LoadedCols::new())
			.insert_resource(Generators::new(0))
			.add_event::<ColUnloadEvent>()
			.add_systems(Update, update_load_area)
			.add_systems(Update, load_order)
			.add_systems(Update, pull_orders)
		;
	}
}