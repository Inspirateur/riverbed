mod terrain_gen;
mod debug_gen;
mod earth_gen;
mod load_area;
mod load_orders;
mod voxel_world;
mod realm;
mod chunk;
mod chunk_shape;
mod pos;
mod utils;
pub use realm::*;
pub use voxel_world::*;
pub use chunk::*;
pub use chunk_shape::*;
pub use pos::*;
pub use load_area::{LoadArea, RenderDistance, range_around};
pub use load_orders::{LoadOrders, ColUnloadEvent};
use bevy::{app::Startup, ecs::schedule::{apply_deferred, IntoSystemConfigs, SystemSet}, prelude::{Plugin, Update}};
use crate::agents::PlayerSpawn;
use self::{load_orders::{
	assign_load_area, on_render_distance_change, process_unload_orders, update_load_area
}, terrain_gen::setup_gen_thread};
use lazy_static::lazy_static;
pub const CHUNK_S1: usize = 62;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNK_S3: usize = CHUNK_S1.pow(3);
pub const CHUNK_PADDED_S1: usize = CHUNK_S1 + 2;
pub const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub const MAX_HEIGHT: usize = 496;
pub const MAX_GEN_HEIGHT: usize = 400;
pub const WATER_H: i32 = 61;
pub const Y_CHUNKS: usize = MAX_HEIGHT/CHUNK_S1;

lazy_static! {
    pub static ref CHUNK_SHAPE: YFirstShape = YFirstShape::new();
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct LoadAreaAssigned;

pub struct GenPlugin;

impl Plugin for GenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.insert_resource(LoadOrders::new())
			.add_event::<ColUnloadEvent>()
			.add_systems(Startup, setup_gen_thread)
			.add_systems(Startup, (assign_load_area, apply_deferred).chain().in_set(LoadAreaAssigned).after(PlayerSpawn))
			.add_systems(Update, update_load_area)
			.add_systems(Update, on_render_distance_change)
			.add_systems(Update, process_unload_orders)
		;
	}
}