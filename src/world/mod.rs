mod load_area;
mod load_orders;
mod voxel_world;
mod realm;
mod chunk;
mod pos;
mod utils;

pub use realm::*;
pub use voxel_world::*;
pub use chunk::*;
pub use pos::*;
pub use load_area::{PlayerArea, RenderDistance, range_around};
pub use load_orders::{LoadOrders, ColUnloadEvent, BlockEntities};
use bevy::{app::Startup, ecs::schedule::{ApplyDeferred, IntoScheduleConfigs, SystemSet}, prelude::{Plugin, Update}};
use crate::{agents::PlayerSpawn, terrain::setup_gen_thread};
use self::load_orders::{
	assign_load_area, on_render_distance_change, process_unload_orders, update_load_area
};
pub const CHUNK_S1: usize = 62;
pub const CHUNK_S2: usize = CHUNK_S1.pow(2);
pub const CHUNKP_S1: usize = CHUNK_S1 + 2;
pub const CHUNKP_S2: usize = CHUNKP_S1.pow(2);
pub const CHUNKP_S3: usize = CHUNKP_S1.pow(3);
pub const CHUNK_S1I: i32 = CHUNK_S1 as i32;

pub const MAX_HEIGHT: usize = 496;
pub const MAX_GEN_HEIGHT: usize = 400;
pub const WATER_H: i32 = 61;
pub const Y_CHUNKS: usize = MAX_HEIGHT/CHUNK_S1;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct LoadAreaAssigned;

pub struct GenPlugin;

impl Plugin for GenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
		app
			.insert_resource(LoadOrders::new())
			.insert_resource(BlockEntities::default())
			.add_event::<ColUnloadEvent>()
			.add_systems(Startup, setup_gen_thread)
			.add_systems(Startup, (assign_load_area, ApplyDeferred).chain().in_set(LoadAreaAssigned).after(PlayerSpawn))
			.add_systems(Update, update_load_area)
			.add_systems(Update, on_render_distance_change)
			.add_systems(Update, process_unload_orders)
		;
	}
}