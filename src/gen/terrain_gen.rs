use crate::gen::earth_gen::Earth;
use crate::blocks::Blocks;
use bevy::ecs::system::Res;
use bevy::prelude::Resource;
use bevy::tasks::AsyncComputeTaskPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::yield_now;
use super::LoadOrders;

#[derive(Resource)]
pub struct Seed(pub u32);

pub fn setup_gen_thread(blocks: Res<Blocks>, seed: Res<Seed>, load_orders: Res<LoadOrders>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let chunks = Arc::clone(&blocks.chunks);
    let seed_value = seed.0;
    let load_orders = Arc::clone(&load_orders.to_generate);
    thread_pool.spawn(
        async move {
            let gen = Earth::new(seed_value, HashMap::new());
            let world = Blocks::new_with(chunks);
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