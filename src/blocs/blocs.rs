use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, BuildHasher};
use bevy::prelude::{Resource, Vec3};
use indexmap::IndexSet;
use super::{
    CHUNK_S1, Y_CHUNKS,  MAX_HEIGHT, ChunkedPos, Chunk, ChunkPos, ColedPos, Realm, Bloc,
    ColPos, BlocPos, BlocPos2d, chunked
};

pub struct BlocRayCastHit {
    pub pos: BlocPos,
    pub normal: Vec3,
}

pub trait IndexSetUtils<V> {
    fn pop_front(&mut self) -> Option<V>;
}

impl<V: Eq + PartialEq + Hash + Clone> IndexSetUtils<V> for IndexSet<V> {
    fn pop_front(&mut self) -> Option<V> {
        let value = self.first().cloned()?;
        self.shift_remove(&value);
        Some(value)
    }
}

#[derive(Resource)]
pub struct Blocs {
    pub chunks: HashMap<ChunkPos, Chunk>,
    // using index map because we want to preserve insertion order here
    pub changes: IndexSet<ChunkPos>,
    pub tracking: HashSet<ChunkPos>,
}

impl Blocs {
    pub fn new() -> Self {
        Blocs {
            chunks: HashMap::new(),
            changes: IndexSet::new(),
            tracking: HashSet::new(),
        }
    }

    pub fn set_bloc(&mut self, pos: BlocPos, bloc: Bloc) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        self.mark_change(chunk_pos, chunked_pos);
        self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new()).set(chunked_pos, bloc);
    }

    pub fn set_bloc_safe(&mut self, pos: BlocPos, bloc: Bloc) {
        if pos.y < 0 || pos.y >= MAX_HEIGHT as i32 { return; }
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        self.mark_change(chunk_pos, chunked_pos);
        self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new()).set(chunked_pos, bloc);
    }

    pub fn set_yrange(&mut self, col_pos: ColPos, (x, z): ColedPos, top: i32, mut height: usize, bloc: Bloc) {
        // BYPASSES CHANGE DETECTION, used by terrain generation to efficiently fill columns of blocs
        let (mut cy, mut dy) = chunked(top);
        while height > 0 && cy >= 0 {
            let chunk_pos = ChunkPos { x: col_pos.x, y: cy, z: col_pos.z, realm: col_pos.realm};
            let h = height.min(dy+1);
            self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new()).set_yrange((x, dy, z), h, bloc);
            height -= h;
            cy -= 1;
            dy = CHUNK_S1-1;
        }
    }

    pub fn set_if_empty(&mut self, pos: BlocPos, bloc: Bloc) {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        if self.chunks.entry(chunk_pos)
            .or_insert_with(|| Chunk::new())
            .set_if_empty(chunked_pos, bloc) 
        {
            self.mark_change(chunk_pos, chunked_pos);
        }
    }
    
    pub fn get_block(&self, pos: BlocPos) -> Bloc {
        let (chunk_pos, chunked_pos) = <(ChunkPos, ChunkedPos)>::from(pos);
        match self.chunks.get(&chunk_pos) {
            None => Bloc::default(),
            Some(chunk) => chunk.get(chunked_pos).clone()
        }
    }

    pub fn copy_column(&self, buffer: &mut [Bloc], chunk_pos: ChunkPos, (x, z): ColedPos) {
        let Some(chunk) = self.chunks.get(&chunk_pos) else {
            return;
        };
        chunk.copy_column(buffer, (x, z));
    }

    pub fn get_block_chunked(&self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos) -> Bloc {
        match self.chunks.get(&chunk_pos) {
            None => Bloc::default(),
            Some(chunk) => chunk.get(chunked_pos).clone()
        }
    }

    pub fn get_block_safe(&self, pos: BlocPos) -> Bloc {
        if pos.y < 0 || pos.y >= MAX_HEIGHT as i32 {
            Bloc::Air
        } else {
            self.get_block(pos)
        }
    }

    pub fn top_block(&self, pos: BlocPos2d) -> (Bloc, i32) {
        let (col_pos, pos2d) = pos.into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk_pos = ChunkPos {
                x: col_pos.x,
                y,
                z: col_pos.z,
                realm: col_pos.realm
            };
            if let Some(chunk) = self.chunks.get(&chunk_pos) {
                let (bloc, bloc_y) = chunk.top(pos2d);
                if *bloc != Bloc::default() {
                    return (bloc.clone(), y*CHUNK_S1 as i32 + bloc_y as i32);
                }
            }
        }
        (Bloc::default(), 0)
    }

    pub fn is_col_loaded(&self, player_pos: Vec3, realm: Realm) -> bool {
        let (chunk_pos, _): (ChunkPos, _) = <BlocPos>::from((player_pos, realm)).into();
        for y in (0..Y_CHUNKS as i32).rev() {
            let chunk = ChunkPos { x: chunk_pos.x, y, z: chunk_pos.z, realm: chunk_pos.realm };
            if self.chunks.contains_key(&chunk) {
                return true;
            }
        }
        false
    }
    
    pub fn unload_col(&mut self, col: ColPos) {
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {x: col.x, y, z: col.z, realm: col.realm };
            self.chunks.remove(&chunk_pos);
            self.changes.remove(&chunk_pos);
            self.tracking.remove(&chunk_pos);
        }
    }

    pub fn register(&mut self, col: ColPos) {
        // Used by terrain generation to batch register chunks for efficiency
        for y in 0..Y_CHUNKS as i32 {
            let chunk_pos = ChunkPos {x: col.x, y, z: col.z, realm: col.realm };
            self.tracking.insert(chunk_pos);
            if self.chunks.contains_key(&chunk_pos) {
                self.changes.insert(chunk_pos);
            }
        }
    }

    fn _mark_change_single(&mut self, chunk_pos: ChunkPos) {
        if self.tracking.contains(&chunk_pos) {
            self.changes.insert(chunk_pos);
        }
    }

    fn border_sign(coord: usize) -> i32 {
        if coord == 0 { -1 } else if coord == CHUNK_S1 -1 { 1 } else { 0 }
    }

    fn mark_change(&mut self, chunk_pos: ChunkPos, chunked_pos: ChunkedPos) {
        self._mark_change_single(chunk_pos);
        // register change for neighboring chunks
        let border_sign_x = Blocs::border_sign(chunked_pos.0); 
        if border_sign_x != 0 {
            let mut neighbor = chunk_pos;
            neighbor.x += border_sign_x;
            self._mark_change_single(neighbor);
        }
        let border_sign_y = Blocs::border_sign(chunked_pos.1); 
        if border_sign_y != 0 {
            let mut neighbor = chunk_pos;
            neighbor.y += border_sign_y;
        	if neighbor.y >= 0 && neighbor.y < Y_CHUNKS as i32 {
                self._mark_change_single(neighbor);
            }
        }
        let border_sign_z = Blocs::border_sign(chunked_pos.2); 
        if border_sign_z != 0 {
            let mut neighbor = chunk_pos;
            neighbor.z += border_sign_z;
            self._mark_change_single(neighbor);
        }
    }

    pub fn raycast(&self, realm: Realm, start: Vec3, dir: Vec3, dist: f32) -> Option<BlocRayCastHit> {
        let mut pos = BlocPos {
            realm, 
            x: start.x.floor() as i32,
            y: start.y.floor() as i32,
            z: start.z.floor() as i32,
        };
        let mut last_pos;
        let sx = dir.x.signum() as i32;
        let sy = dir.y.signum() as i32;
        let sz = dir.z.signum() as i32;
        if sx == 0 && sy == 0 && sz == 0 {
            return None;
        }
        let next_x = (pos.x + sx.max(0)) as f32;
        let next_y = (pos.y + sy.max(0)) as f32;
        let next_z = (pos.z + sz.max(0)) as f32;
        let mut t_max_x = (next_x - start.x) / dir.x;
        let mut t_max_y = (next_y - start.y) / dir.y;
        let mut t_max_z = (next_z - start.z) / dir.z;
        let slope_x = 1./dir.x.abs();
        let slope_y = 1./dir.y.abs();
        let slope_z = 1./dir.z.abs();
        loop {
            last_pos = pos.clone();
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    if t_max_x >= dist { return None };
                    pos.x += sx;
                    t_max_x += slope_x;
                } else {
                    if t_max_z >= dist { return None };
                    pos.z += sz;
                    t_max_z += slope_z;
                }
            } else {
                if t_max_y < t_max_z {
                    if t_max_y >= dist { return None };
                    pos.y += sy;
                    t_max_y += slope_y;
                } else {
                    if t_max_z >= dist { return None };
                    pos.z += sz;
                    t_max_z += slope_z;
                }
            }
            if self.get_block_safe(pos).targetable() {
                return Some(BlocRayCastHit {
                    pos, normal: Vec3 { 
                        x: (last_pos.x-pos.x) as f32, 
                        y: (last_pos.y-pos.y) as f32, 
                        z: (last_pos.z-pos.z) as f32 
                    }
                });
            }
        }
    }
}