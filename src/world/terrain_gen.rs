use crate::world::earth_gen::Earth;
use crate::world::VoxelWorld;
use crate::WorldRng;
use bevy::ecs::system::Res;
use bevy::tasks::AsyncComputeTaskPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::yield_now;
use super::LoadOrders;

pub fn setup_gen_thread(blocks: Res<VoxelWorld>, world_rng: Res<WorldRng>, load_orders: Res<LoadOrders>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let chunks = Arc::clone(&blocks.chunks);
    let seed_value = world_rng.seed;
    let load_orders = Arc::clone(&load_orders.to_generate);
    thread_pool.spawn(
        async move {
            let gen = Earth::new(seed_value as u32, HashMap::new());
            let world = VoxelWorld::new_with(chunks);
            loop {
                let Some((col_pos, _)) = load_orders.try_write_arc().and_then(|mut ld| ld.pop()) else {
                    yield_now();
                    continue;
                };
                gen.gen(&world, col_pos);
                world.mark_change_col(col_pos);
            }
        }
    ).detach();
}