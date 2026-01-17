//! Client-side movement using shared physics.
//!
//! This module provides client-side movement prediction using the same physics
//! code as the server. This ensures deterministic behavior and minimal drift
//! during reconciliation.

use bevy::{prelude::*, time::Timer};
use itertools::iproduct;
use shared::physics::{player_step::apply_player_input_step, MovementMode, PhysicsState};
use shared::world::{pos::pos3d::BlockPos, realm::Realm, BlockAccess};
use shared::{FLY_SPEED, WALK_SPEED};

use crate::network::buffered_client::CurrentFrameInputs;
use crate::render::FpsCam;
use crate::world::ClientWorldMap;
use crate::Block;

use super::PlayerControlled;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_stepped_block)
            .add_systems(Update, apply_shared_physics);
    }
}

/// Block the entity is standing on (used for footstep sounds and friction info)
#[derive(Component)]
pub struct SteppingOn(pub Block);

/// Marker component for walking movement mode
#[derive(Component)]
pub struct Walking;

/// Marker component for flying movement mode
#[derive(Component)]
pub struct FreeFly;

/// Movement speed component
#[derive(Component)]
pub struct Speed(pub f32);

/// Jump state component
#[derive(Component)]
pub struct Jumping {
    pub force: f32,
    pub cd: Timer,
    pub intent: bool,
}

/// Crouch state component
#[derive(Component)]
pub struct Crouching(pub bool);

/// Axis-aligned bounding box for collision detection
#[derive(Component)]
pub struct AABB(pub Vec3);

/// Desired movement direction and speed (set by input system)
#[derive(Component)]
pub struct Heading(pub Vec3);

/// Current velocity vector
#[derive(Component)]
pub struct Velocity(pub Vec3);

/// Gravity strength for the entity
#[derive(Component)]
pub struct Gravity(pub f32);

// Helper functions for stepped block detection (used for footstep sounds)
fn extent(v: f32, size: f32) -> Vec<i32> {
    let start = v.floor() as i32;
    let end = (size + v).floor() as i32;
    if size > 0.0 {
        (start..=end).collect()
    } else {
        (end..=start).rev().collect()
    }
}

fn blocks_perp_y(pos: Vec3, realm: Realm, aabb: &AABB) -> impl Iterator<Item = BlockPos> {
    iproduct!(extent(pos.x, aabb.0.x), extent(pos.z, aabb.0.z)).map(move |(x, z)| BlockPos {
        x,
        y: pos.y.floor() as i32,
        z,
        realm,
    })
}

/// Updates the SteppingOn component to track what block the player is standing on.
/// This is used for footstep sounds and other effects.
fn update_stepped_block(
    blocks: Res<ClientWorldMap>,
    mut query: Query<(&Transform, &Realm, &AABB, &mut SteppingOn)>,
) {
    for (transform, realm, aabb, mut stepping_on) in query.iter_mut() {
        let below = transform.translation + Vec3::new(0., -0.01, 0.);
        let mut closest_block = Block::Air;
        let mut min_dist = f32::INFINITY;
        for block_pos in blocks_perp_y(below, *realm, aabb) {
            let block = blocks.get_block(block_pos);
            if block.is_traversable() {
                continue;
            }
            let dist = (below.x - block_pos.x as f32).abs() - (below.y - block_pos.y as f32).abs();
            if dist < min_dist {
                min_dist = dist;
                closest_block = block;
            }
        }
        stepping_on.0 = closest_block;
    }
}

/// Applies the shared physics simulation to the local player.
///
/// This system uses the same `apply_player_input_step` function that the server uses,
/// ensuring that client-side prediction produces identical results to the server's
/// authoritative simulation (given the same inputs and world state).
fn apply_shared_physics(
    mut commands: Commands,
    time: Res<Time>,
    world: Res<ClientWorldMap>,
    frame_inputs: Res<CurrentFrameInputs>,
    camera_query: Query<&Transform, With<FpsCam>>,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &Realm,
            Option<&FreeFly>,
        ),
        (With<PlayerControlled>, Without<FpsCam>),
    >,
) {
    let Ok((entity, mut transform, mut velocity, realm, free_fly_opt)) = player_query.single_mut()
    else {
        return;
    };

    // Skip if no delta time (first frame)
    if frame_inputs.0.delta_ms == 0 {
        return;
    }

    // Get camera transform for movement orientation
    let camera_transform = camera_query.single().copied().unwrap_or_default();

    // Build current physics state
    let current_mode = if free_fly_opt.is_some() {
        MovementMode::Flying
    } else {
        MovementMode::Walking
    };

    let state = PhysicsState {
        position: transform.translation,
        velocity: velocity.0,
        movement_mode: current_mode,
        realm: *realm,
        on_ground: false, // Will be computed by physics
    };

    // Apply shared physics step
    let delta_seconds = time.delta_secs();
    let step = apply_player_input_step(
        &*world,
        &state,
        &frame_inputs.0.inputs,
        &camera_transform,
        delta_seconds,
    );

    // Update transform and velocity from physics result
    transform.translation = step.position;
    velocity.0 = step.velocity;

    // Sync movement mode ECS components if changed
    let new_is_flying = step.movement_mode == MovementMode::Flying;
    let was_flying = free_fly_opt.is_some();

    if new_is_flying != was_flying {
        if new_is_flying {
            commands.entity(entity).remove::<Walking>().insert(FreeFly);
            commands.entity(entity).insert(Speed(FLY_SPEED));
        } else {
            commands.entity(entity).remove::<FreeFly>().insert(Walking);
            commands.entity(entity).insert(Speed(WALK_SPEED));
        }
    }
}
