mod terrain_gen;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_orders;
pub use terrain_gen::Generators;
pub use load_area::{LoadArea, RenderDistance};
pub use load_orders::{LoadOrders, ColUnloadEvent};
use bevy::prelude::{Plugin, Update};
use self::load_orders::{
	assign_load_area, update_load_area, on_render_distance_change, process_unload_orders, process_load_order
};

pub struct GenPlugin;

impl Plugin for GenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.insert_resource(LoadOrders::new())
			.insert_resource(Generators::new(0))
			.add_event::<ColUnloadEvent>()
			.add_systems(Update, assign_load_area)
			.add_systems(Update, update_load_area)
			.add_systems(Update, on_render_distance_change)
			.add_systems(Update, process_unload_orders)
			.add_systems(Update, process_load_order)
		;
	}
}