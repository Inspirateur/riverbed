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
use self::{load_area::{update_load_area, compute_load_orders}, load_cols::{process_unload_orders, process_load_order}};

pub struct GenPlugin;

impl Plugin for GenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.insert_resource(LoadedCols::new())
			.insert_resource(Generators::new(0))
			.add_event::<ColUnloadEvent>()
			.add_systems(Update, update_load_area)
			.add_systems(Update, compute_load_orders)
			.add_systems(Update, process_unload_orders)
			.add_systems(Update, process_load_order)
		;
	}
}