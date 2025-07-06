use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::log::trace;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver, Sender};
use crate::generation::Earth;
use crate::logging::LogData;
use crate::world::{ColPos, ColUnloadEvent, PlayerCol, Realm, VoxelWorld};
use crate::WorldRng;

pub fn setup_load_thread(mut commands: Commands, world: Res<VoxelWorld>, world_rng: Res<WorldRng>) {
    let (player_pos_sender, player_pos_recv) = unbounded::<PlayerColumnUpdate>();
    commands.insert_resource(PlayerColumnUpdateSender(player_pos_sender));
    let (unload_sender, unload_recv) = unbounded::<ColPos>();
    commands.insert_resource(ColUnloadsReciever(unload_recv));
    let thread_pool = AsyncComputeTaskPool::get();
    let load_world = world.clone();
    let seed_value = world_rng.seed;

    thread_pool.spawn(
        async move {
            let terrain_gen = Earth::new(seed_value as u32, HashMap::new());
            // local copy of players positions
            let mut players_pos = HashMap::new();
            // keeps track of which players see which columns
            let mut player_cols: HashMap<ColPos, HashSet<u32>> = HashMap::new();
            // the list of all columns that must be generated
            let mut to_load: Vec<ColPos> = Vec::new();
            loop {
                // Queue load orders and unload terrain based on incoming player positions and RENDER_DISTANCE
                while let Ok(player_pos_update) = player_pos_recv.try_recv() {
                    let player_area_diff = player_pos_update.new_col.player_area_diff(player_pos_update.old_col_opt);
                    players_pos.insert(player_pos_update.id, player_pos_update.new_col);
                    for col in player_area_diff.exclusive_in_other {
                        if let Some(cols) = player_cols.get_mut(&col) {
                            // unregister the player from the column
                            cols.remove(&player_pos_update.id);
                            if cols.is_empty() {
                                // if no players are left in the column, we can unload it
                                load_world.unload_col(col);
                                trace!("{}", LogData::ColUnloaded(col));
                                if unload_sender.send(col).is_err() {
                                    panic!("ColUnloadsReciever channel is closed");
                                }
                            }
                        }
                    }
                    for col in player_area_diff.exclusive_in_self {
                        // register the player in the column
                        let players = player_cols.entry(col).or_default();
                        if players.is_empty() {
                            to_load.push(col);
                        }
                        players.insert(player_pos_update.id);
                    }
                }
                // Generate the closest column to any player
                if let Some((closest_idx, _closest_col)) = to_load
                    .iter()
                    .enumerate()
                    .min_by_key(|(_i, col)| 
                        players_pos.values()
                            .map(|player_col| (col.x - player_col.x).abs() + (col.z - player_col.z).abs())
                            .min()
                    )
                {
                    let col = to_load.remove(closest_idx);
                    terrain_gen.generate(&load_world, col);
                    trace!("{}", LogData::ColGenerated(col));
                    load_world.mark_change_col(col);
                }
            }
        }
    ).detach();
}

pub fn assign_player_col(
    mut commands: Commands, 
    sender: Res<PlayerColumnUpdateSender>, 
    player_query: Query<(Entity, &Transform, &Realm), Without<PlayerCol>>,
) {
    for (player, transform, realm) in player_query.iter() {
        let col = ColPos::from((transform.translation, *realm));
        commands.entity(player).insert(PlayerCol(col));
        let update = PlayerColumnUpdate {
            id: player.index(),
            old_col_opt: None,
            new_col: col,
        };
        if sender.0.send(update).is_err() {
            panic!("PlayerColumnUpdateSender channel is closed");
        }
    }
}

pub fn send_player_pos_update(
    sender: Res<PlayerColumnUpdateSender>, 
    mut player_query: Query<(Entity, &Transform, &Realm, &mut PlayerCol)>,
) {
    for (player, transform, realm, mut player_col) in player_query.iter_mut() {
        let new_col = ColPos::from((transform.translation, *realm));
        if player_col.0 != new_col {
            // send the update only if the column has changed
            let update = PlayerColumnUpdate {
                id: player.index(),
                old_col_opt: Some(player_col.0),
                new_col,
            };
            if sender.0.send(update).is_err() {
                panic!("PlayerColumnUpdateSender channel is closed");
            }
            player_col.0 = new_col;
        }
    }
}

#[derive(Resource)]
pub struct PlayerColumnUpdateSender(pub Sender<PlayerColumnUpdate>);

pub struct PlayerColumnUpdate {
    id: u32,
    old_col_opt: Option<ColPos>,
    new_col: ColPos,
}


#[derive(Resource)]
pub struct ColUnloadsReciever(pub Receiver<ColPos>);

pub fn on_unload_col(
    unload_cols: Res<ColUnloadsReciever>,
    mut unload_event: EventWriter<ColUnloadEvent>,
) {
    while let Ok(col) = unload_cols.0.try_recv() {
        unload_event.write(ColUnloadEvent(col));
    }
}