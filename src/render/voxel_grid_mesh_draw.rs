use crate::block::Face;
use crate::render::texture_array::{BlockTextureArray, TextureMap};
use crate::render::voxel_grid_mesh_thread::{
    GridColliderOutput, GridMeshOutput, spawn_grid_mesh_thread,
};
use crate::world::{CHUNK_S1, GridChunkPos, VoxelGrid};
use avian3d::prelude::{Collider, Dominance, Friction, RigidBody};
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use crossbeam::channel::{Receiver, unbounded};
use itertools::Itertools;
use std::collections::HashMap;

use super::BlockTexState;

pub struct VoxelGridPlugin;

impl Plugin for VoxelGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pull_grid_meshes, pull_grid_colliders).run_if(in_state(BlockTexState::Mapped)),
        );
    }
}

#[derive(Component)]
pub struct GridMeshReceiver(pub Receiver<GridMeshOutput>);

#[derive(Component)]
pub struct GridColliderReceiver(pub Receiver<GridColliderOutput>);

/// Tracks the visual and collider child entities owned by a `VoxelGrid` root,
/// keyed by chunk position. Pull systems use it to update or despawn children
/// when chunk geometry changes.
#[derive(Component, Default)]
pub struct GridChildEntities {
    mesh: HashMap<(GridChunkPos, Face), Entity>,
    collider: HashMap<GridChunkPos, Entity>,
}

/// Spawn a movable voxel grid: creates the worker, runs the `build` closure to
/// fill blocks, spawns the rigidbody root, and queues every populated chunk
/// for meshing. Must be called while `BlockTexState::Mapped` is active so the
/// worker captures a populated texture map.
pub fn spawn_voxel_grid(
    commands: &mut Commands,
    texture_map: &TextureMap,
    transform: Transform,
    build: impl FnOnce(&mut VoxelGrid),
) -> Entity {
    let (order_sender, order_receiver) = unbounded::<GridChunkPos>();
    let mut grid = VoxelGrid::new(order_sender.clone());
    build(&mut grid);
    let chunks = grid.chunks.clone();
    let (mesh_receiver, collider_receiver) =
        spawn_grid_mesh_thread(chunks.clone(), texture_map.0.clone(), order_receiver);
    let entity = commands
        .spawn((
            grid,
            RigidBody::Dynamic,
            Friction::new(1.0),
            // Outweigh the player so resting/walking on a grid doesn't push
            // it around. Grid-vs-grid stays balanced (equal dominance).
            Dominance(10),
            transform,
            Visibility::default(),
            GridMeshReceiver(mesh_receiver),
            GridColliderReceiver(collider_receiver),
            GridChildEntities::default(),
        ))
        .id();
    for entry in chunks.iter() {
        let _ = order_sender.send(*entry.key());
    }
    entity
}

fn pull_grid_meshes(
    mut commands: Commands,
    mut grids: Query<(Entity, &GridMeshReceiver, &mut GridChildEntities)>,
    mut mesh_query: Query<&mut Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
    block_tex_array: Res<BlockTextureArray>,
) {
    for (grid_entity, recv, mut tracker) in grids.iter_mut() {
        // Coalesce duplicates so older geometry is dropped.
        let received: Vec<_> = recv.0.try_iter().collect();
        for (mesh_opt, chunk_pos, face) in received
            .into_iter()
            .rev()
            .unique_by(|(_, pos, face)| (*pos, *face))
        {
            let key = (chunk_pos, face);
            let Some(mesh) = mesh_opt else {
                if let Some(ent) = tracker.mesh.remove(&key) {
                    commands.entity(ent).despawn();
                }
                continue;
            };
            if let Some(&ent) = tracker.mesh.get(&key) {
                if let Ok(mut handle) = mesh_query.get_mut(ent) {
                    handle.0 = meshes.add(mesh);
                    continue;
                }
            }
            let chunk_aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_S1 as f32));
            let child = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(block_tex_array.0.clone()),
                    Transform::from_translation(Vec3::from(chunk_pos)),
                    NoFrustumCulling,
                    chunk_aabb,
                ))
                .id();
            commands.entity(grid_entity).add_child(child);
            tracker.mesh.insert(key, child);
        }
    }
}

fn pull_grid_colliders(
    mut commands: Commands,
    mut grids: Query<(Entity, &GridColliderReceiver, &mut GridChildEntities)>,
    mut collider_query: Query<&mut Collider>,
) {
    for (grid_entity, recv, mut tracker) in grids.iter_mut() {
        let received: Vec<_> = recv.0.try_iter().collect();
        for (collider_opt, chunk_pos) in received.into_iter().rev().unique_by(|(_, pos)| *pos) {
            let Some(collider) = collider_opt else {
                if let Some(ent) = tracker.collider.remove(&chunk_pos) {
                    commands.entity(ent).despawn();
                }
                continue;
            };
            if let Some(&ent) = tracker.collider.get(&chunk_pos) {
                if let Ok(mut handle) = collider_query.get_mut(ent) {
                    *handle = collider;
                    continue;
                }
            }
            let child = commands
                .spawn((
                    collider,
                    Transform::from_translation(Vec3::from(chunk_pos)),
                ))
                .id();
            commands.entity(grid_entity).add_child(child);
            tracker.collider.insert(chunk_pos, child);
        }
    }
}
