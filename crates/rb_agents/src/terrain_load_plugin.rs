use bevy::ecs::entity::EntityIndex;
use bevy::log::trace;
use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender, unbounded};
use rb_generation::TerrainGenerator;
use rb_logging::LogData;
use rb_world::{
    BlockEntities, ChunkPos2d, ColUnloadEvent, PlayerCol, Realm, StructureTrait, VoxelWorld,
    WorldRng, player_area_diff, unload_block_entities,
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Condvar, Mutex},
};
const THREAD_COUNT: usize = 1;

fn thread_owns_col(col: ChunkPos2d, thread_id: u32) -> bool {
    (col.x + col.z).rem_euclid(THREAD_COUNT as i32) == thread_id as i32
}

/// A per-thread, per-player mailbox: sending a position overwrites any not-yet-read
/// position for that same player, so a lagging reader always catches up to the latest.
pub struct PosMailbox {
    pending: Mutex<HashMap<EntityIndex, ChunkPos2d>>,
    signal: Condvar,
}

impl PosMailbox {
    fn new() -> Self {
        PosMailbox {
            pending: Mutex::new(HashMap::new()),
            signal: Condvar::new(),
        }
    }

    fn send(&self, id: EntityIndex, pos: ChunkPos2d) {
        self.pending.lock().unwrap().insert(id, pos);
        self.signal.notify_one();
    }

    /// Blocks until at least one update is pending, then drains all of them.
    fn recv_all(&self) -> HashMap<EntityIndex, ChunkPos2d> {
        let mut guard = self.pending.lock().unwrap();
        while guard.is_empty() {
            guard = self.signal.wait(guard).unwrap();
        }
        std::mem::take(&mut *guard)
    }

    /// Drains whatever is currently pending without blocking (may be empty).
    fn try_recv_all(&self) -> HashMap<EntityIndex, ChunkPos2d> {
        std::mem::take(&mut *self.pending.lock().unwrap())
    }
}
pub struct TerrainLoadPlugin;

impl Plugin for TerrainLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ColUnloadEvent>()
            .insert_resource(BlockEntities::default())
            .add_systems(Startup, setup_load_thread)
            .add_systems(Update, send_player_pos_update)
            .add_systems(Update, assign_player_col)
            .add_systems(Update, on_unload_col)
            .add_systems(Update, unload_block_entities);
    }
}

pub fn setup_load_thread(mut commands: Commands, world: Res<VoxelWorld>, world_rng: Res<WorldRng>) {
    let mailboxes: Vec<Arc<PosMailbox>> = (0..THREAD_COUNT)
        .map(|_| Arc::new(PosMailbox::new()))
        .collect();
    commands.insert_resource(PlayerColumnUpdateSender(mailboxes.clone()));
    let (unload_sender, unload_recv) = unbounded::<ChunkPos2d>();
    commands.insert_resource(ColUnloadsReciever(unload_recv));
    let seed_value = world_rng.seed;

    for (i, mailbox) in mailboxes.into_iter().enumerate() {
        let unload_sender = unload_sender.clone();
        let load_world = world.clone();
        std::thread::spawn(move || {
            generation_thread(seed_value, mailbox, unload_sender, load_world, i as u32);
        });
    }
}

fn generation_thread(
    seed: u64,
    mailbox: Arc<PosMailbox>,
    unload_sender: Sender<ChunkPos2d>,
    load_world: VoxelWorld,
    thread_id: u32,
) {
    let mut terrain_gen = TerrainGenerator::new(seed, Path::new("assets"));
    // local copy of players positions
    let mut players_pos = HashMap::new();
    // keeps track of which players see which columns
    let mut players_by_col: HashMap<ChunkPos2d, HashSet<EntityIndex>> = HashMap::new();
    // the list of all columns that must be generated
    let mut to_load: Vec<ChunkPos2d> = Vec::new();
    let mut structure_map: HashMap<ChunkPos2d, Vec<Box<dyn StructureTrait>>> = HashMap::new();
    loop {
        let loader_span = info_span!("loader", name = "loading 1 column").entered();
        let update_span = info_span!("loader", name = "receiving player update").entered();
        // Queue load orders and unload terrain based on incoming player positions and RENDER_DISTANCE
        // If to_load is empty, we block on player position updates to not waste resources
        let updates = if to_load.len() == 0 {
            mailbox.recv_all()
        } else {
            mailbox.try_recv_all()
        };
        update_span.exit();
        let processing_span = info_span!("loader", name = "processing player update").entered();
        for (id, new_col) in updates {
            let old_col_opt = players_pos.get(&id).copied();
            if old_col_opt == Some(new_col) {
                continue;
            }
            // Compute the difference in player area
            let area_diff = player_area_diff(&new_col, old_col_opt);
            players_pos.insert(id, new_col);
            // Handle columns that are no longer in the player's area
            for col in area_diff.exclusive_in_other {
                let Some(cols) = players_by_col.get_mut(&col) else {
                    continue;
                };
                cols.remove(&id);
                if !cols.is_empty() {
                    continue;
                }
                // we remove it from the list of columns that should be loaded in the world
                load_world.loaded_columns.remove(&col);
                if let Some(i) = to_load.iter().position(|c| *c == col) {
                    // the chunk was still in the load queue we remove it
                    to_load.swap_remove(i);
                    // even in this case we still need to unload the column after because
                    // it could have received blocks from neighboring columns generation
                }
                load_world.unload_col(col);
                if unload_sender.send(col).is_err() {
                    // This means the game is shutting down, so we break the loop
                    warn!("ColUnloadsReciever channel is closed, stopping terrain thread");
                    break;
                }
            }
            // Handle columns that are new in the player's area
            // Only track columns owned by this thread: otherwise every thread would
            // duplicate bookkeeping (and the O(n) closest-column search below) for the
            // entire render-distance area instead of just its own share of it.
            for col in &area_diff.exclusive_in_self {
                if !thread_owns_col(*col, thread_id) {
                    continue;
                }
                let players = players_by_col.entry(*col).or_default();
                if players.is_empty() {
                    to_load.push(*col);
                }
                players.insert(id);
            }
        }
        processing_span.exit();
        if to_load.is_empty() {
            continue;
        }
        let closest_span = info_span!("loader", name = "finding closest column").entered();
        // Generate the closest column to any player
        let (closest_idx, _closest_col) = to_load
            .iter()
            .enumerate()
            .min_by_key(|(_i, col)| {
                players_pos
                    .values()
                    .map(|player_col| (col.x - player_col.x).abs() + (col.z - player_col.z).abs())
                    .min()
            })
            .unwrap();
        let col = to_load.remove(closest_idx);
        closest_span.exit();
        let generation_span = info_span!("loader", name = "generating terrain").entered();
        let (column, structures) = terrain_gen.generate(col);
        structure_map.insert(col, structures);
        trace!("{}", LogData::ColGenerated(col));
        generation_span.exit();
        let adding_span = info_span!("loader", name = "adding column to world").entered();
        load_world.add_column(col, column);
        adding_span.exit();
        loader_span.exit();
    }
}

pub fn assign_player_col(
    mut commands: Commands,
    sender: Res<PlayerColumnUpdateSender>,
    player_query: Query<(Entity, &Transform, &Realm), Without<PlayerCol>>,
) {
    for (player, transform, realm) in player_query.iter() {
        let col = ChunkPos2d::from((transform.translation, *realm));
        commands.entity(player).insert(PlayerCol(col));
        trace!(
            "{}",
            LogData::PlayerMoved {
                id: player.index_u32(),
                new_col: col
            }
        );
        for mailbox in &sender.0 {
            mailbox.send(player.index(), col);
        }
    }
}

pub fn send_player_pos_update(
    sender: Res<PlayerColumnUpdateSender>,
    mut player_query: Query<(Entity, &Transform, &Realm, &mut PlayerCol)>,
) {
    for (player, transform, realm, mut player_col) in player_query.iter_mut() {
        let new_col = ChunkPos2d::from((transform.translation, *realm));
        if player_col.0 != new_col {
            // send the update only if the column has changed
            trace!(
                "{}",
                LogData::PlayerMoved {
                    id: player.index_u32(),
                    new_col
                }
            );
            for mailbox in &sender.0 {
                mailbox.send(player.index(), new_col);
            }
            player_col.0 = new_col;
        }
    }
}

#[derive(Resource)]
pub struct PlayerColumnUpdateSender(pub Vec<Arc<PosMailbox>>);

#[derive(Resource)]
pub struct ColUnloadsReciever(pub Receiver<ChunkPos2d>);

pub fn on_unload_col(
    unload_cols: Res<ColUnloadsReciever>,
    mut unload_event: MessageWriter<ColUnloadEvent>,
) {
    while let Ok(col) = unload_cols.0.try_recv() {
        unload_event.write(ColUnloadEvent(col));
    }
}
