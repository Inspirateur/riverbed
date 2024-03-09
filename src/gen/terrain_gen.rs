use crate::blocs::pos2d::Pos2d;
use crate::gen::earth_gen::Earth;
use crate::blocs::{Blocs, CHUNK_S1};
use bevy::ecs::system::{Commands, Res};
use bevy::prelude::Resource;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::yield_now;

#[derive(Resource)]
pub struct Seed(pub u32);

#[derive(Resource)]
pub struct LoadOrderSender(pub Sender<Pos2d<CHUNK_S1>>);


pub fn setup_gen_thread(mut commands: Commands, blocs: Res<Blocs>, seed: Res<Seed>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let (load_order_sender, load_order_reciever) = unbounded();
    let chunks = Arc::clone(&blocs.chunks);
    commands.insert_resource(LoadOrderSender(load_order_sender));
    let seed_value = seed.0;
    thread_pool.spawn(
        async move {
            let gen = Earth::new(seed_value, HashMap::new());
            let world = Blocs::new_with(chunks);
            loop {
                let Ok(col_pos) = load_order_reciever.try_recv() else {
                    yield_now();
                    continue;
                };
                gen.gen(&world, col_pos);
                world.mark_change_col(col_pos);
            }
        }
    ).detach();
}