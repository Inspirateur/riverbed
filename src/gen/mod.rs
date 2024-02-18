mod terrain_gen;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_orders;
pub use terrain_gen::Generators;
pub use load_area::{LoadArea, RenderDistance};
pub use load_orders::{LoadOrders, ColUnloadEvent};
use bevy::{app::Startup, ecs::schedule::{apply_deferred, IntoSystemConfigs, SystemSet}, prelude::{Plugin, Update}};
use crate::agents::PlayerSpawn;

use self::load_orders::{
	assign_load_area, on_render_distance_change, process_load_order, process_unload_orders, update_load_area
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct LoadAreaAssigned;

pub struct GenPlugin;

impl Plugin for GenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.insert_resource(LoadOrders::new())
			.insert_resource(Generators::new(0))
			.add_event::<ColUnloadEvent>()
			.add_systems(Startup, (assign_load_area, apply_deferred).chain().in_set(LoadAreaAssigned).after(PlayerSpawn))
			.add_systems(Update, update_load_area)
			.add_systems(Update, on_render_distance_change)
			.add_systems(Update, process_unload_orders)
			.add_systems(Update, process_load_order)
		;
	}
}