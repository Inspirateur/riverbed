//! Shared physics simulation module.
//!
//! This module provides physics simulation code that can be used by both
//! client and server for authoritative movement. The server uses this
//! for authoritative position calculations, and the client uses it for
//! client-side prediction.

use bevy::prelude::*;
use itertools::{iproduct, Itertools};

pub mod player_step;

use crate::block::Block;
use crate::world::block_access::BlockAccess;
use crate::world::pos::pos3d::BlockPos;
use crate::world::realm::Realm;
use crate::{FLY_SPEED, FLY_VERTICAL_SPEED, WALK_SPEED};

/// Player physics constants
pub const PLAYER_GRAVITY: f32 = 50.0;
pub const PLAYER_JUMP_FORCE: f32 = 13.0;
pub const PLAYER_AABB: Vec3 = Vec3::new(0.5, 1.7, 0.5);
pub const ACC_MULT: f32 = 150.0;

/// Represents the movement mode of an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MovementMode {
    #[default]
    Walking,
    Flying,
}

impl MovementMode {
    pub fn speed(&self) -> f32 {
        match self {
            MovementMode::Walking => WALK_SPEED,
            MovementMode::Flying => FLY_SPEED,
        }
    }
}

/// Input state for a single physics tick
#[derive(Debug, Clone, Default)]
pub struct MovementInput {
    /// Horizontal movement direction (normalized), relative to camera
    pub move_direction: Vec3,
    /// Whether the jump/fly-up input is pressed
    pub jump: bool,
    /// Whether the crouch/fly-down input is pressed
    pub crouch: bool,
    /// Camera forward direction (for movement orientation)
    pub camera_forward: Vec3,
    /// Camera right direction (for movement orientation)
    pub camera_right: Vec3,
}

/// Physics state for an entity
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PhysicsState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub movement_mode: MovementMode,
    pub realm: Realm,
    /// Whether the entity is on the ground (for jumping)
    pub on_ground: bool,
}

/// Result of a physics simulation step
#[derive(Debug, Clone)]
pub struct PhysicsStepResult {
    pub new_position: Vec3,
    pub new_velocity: Vec3,
    pub on_ground: bool,
}

/// Compute the extent of positions to check for a given coordinate and size
fn extent(v: f32, size: f32) -> Vec<i32> {
    let start = v.floor() as i32;
    let end = (size + v).floor() as i32;
    if size > 0.0 {
        (start..=end).collect()
    } else {
        (end..=start).rev().collect()
    }
}

/// Get block positions perpendicular to the Y axis (for Y collision checks)
fn blocks_perp_y(pos: Vec3, realm: Realm, aabb: Vec3) -> impl Iterator<Item = BlockPos> {
    iproduct!(extent(pos.x, aabb.x), extent(pos.z, aabb.z)).map(move |(x, z)| BlockPos {
        x,
        y: pos.y.floor() as i32,
        z,
        realm,
    })
}

/// Get block positions perpendicular to the Z axis (for Z collision checks)
fn blocks_perp_z(pos: Vec3, realm: Realm, aabb: Vec3) -> impl Iterator<Item = BlockPos> {
    iproduct!(extent(pos.x, aabb.x), extent(pos.y, aabb.y)).map(move |(x, y)| BlockPos {
        x,
        y,
        z: pos.z.floor() as i32,
        realm,
    })
}

/// Get block positions perpendicular to the X axis (for X collision checks)
fn blocks_perp_x(pos: Vec3, realm: Realm, aabb: Vec3) -> impl Iterator<Item = BlockPos> {
    iproduct!(extent(pos.y, aabb.y), extent(pos.z, aabb.z)).map(move |(y, z)| BlockPos {
        x: pos.x.floor() as i32,
        y,
        z,
        realm,
    })
}

/// Check if the entity is standing on solid ground
pub fn check_on_ground<W: BlockAccess>(
    world: &W,
    position: Vec3,
    realm: Realm,
    aabb: Vec3,
) -> bool {
    let below = position + Vec3::new(0., -0.01, 0.);
    for block_pos in blocks_perp_y(below, realm, aabb) {
        let block = world.get_block_safe(block_pos);
        if !block.is_traversable() {
            return true;
        }
    }
    false
}

/// Get the block the entity is standing on (for friction/slowing calculations)
pub fn get_stepped_block<W: BlockAccess>(
    world: &W,
    position: Vec3,
    realm: Realm,
    aabb: Vec3,
) -> Block {
    let below = position + Vec3::new(0., -0.01, 0.);
    let mut closest_block = Block::Air;
    let mut min_dist = f32::INFINITY;
    for block_pos in blocks_perp_y(below, realm, aabb) {
        let block = world.get_block_safe(block_pos);
        if block.is_traversable() {
            continue;
        }
        let dist = (below.x - block_pos.x as f32).abs() - (below.y - block_pos.y as f32).abs();
        if dist < min_dist {
            min_dist = dist;
            closest_block = block;
        }
    }
    closest_block
}

/// Simulate one physics step for an entity.
///
/// This is the core physics simulation that both client and server use.
/// It handles gravity, collision detection, and movement.
///
/// # Arguments
/// * `world` - Block access for collision detection
/// * `state` - Current physics state
/// * `input` - Movement input for this frame
/// * `delta_seconds` - Time step in seconds
///
/// # Returns
/// The result of the physics step with new position, velocity, and ground state.
pub fn simulate_physics_step<W: BlockAccess>(
    world: &W,
    state: &PhysicsState,
    input: &MovementInput,
    delta_seconds: f32,
) -> PhysicsStepResult {
    let aabb = PLAYER_AABB;
    let mut position = state.position;
    let mut velocity = state.velocity;

    // Calculate world-space movement direction from input
    let forward_horizontal =
        Vec3::new(input.camera_forward.x, 0.0, input.camera_forward.z).normalize_or_zero();
    let right_horizontal =
        Vec3::new(input.camera_right.x, 0.0, input.camera_right.z).normalize_or_zero();

    // Transform local movement direction to world space
    let world_move_dir = if input.move_direction.length_squared() > 0.0 {
        let local_dir = input.move_direction.normalize();
        forward_horizontal * local_dir.z + right_horizontal * local_dir.x
    } else {
        Vec3::ZERO
    };

    // Calculate heading (desired velocity)
    let speed = state.movement_mode.speed();
    let heading = world_move_dir * speed;

    match state.movement_mode {
        MovementMode::Flying => {
            // Flying mode: direct velocity control, no collision
            velocity.x = heading.x;
            velocity.z = heading.z;
            velocity.y = (input.jump as i32 - input.crouch as i32) as f32 * FLY_VERTICAL_SPEED;

            // In flying mode, skip collision detection entirely
            position += velocity * delta_seconds;

            return PhysicsStepResult {
                new_position: position,
                new_velocity: velocity,
                on_ground: false,
            };
        }
        MovementMode::Walking => {
            // Apply gravity
            velocity.y -= PLAYER_GRAVITY * delta_seconds;

            // Check if on ground for jumping
            let on_ground = check_on_ground(world, position, state.realm, aabb);

            // Handle jumping
            if input.jump && on_ground {
                velocity.y = PLAYER_JUMP_FORCE;
            }

            // Get stepped block for friction/slowing
            let stepped_block = get_stepped_block(world, position, state.realm, aabb);
            let friction = stepped_block.friction();
            let slowing = stepped_block.slowing();

            // Apply slowing to heading
            let slowed_heading = Vec3::new(heading.x * slowing, f32::NAN, heading.z * slowing);

            // Make velocity inch towards heading (X and Z only)
            let diff = Vec3::new(
                slowed_heading.x - velocity.x,
                0.0,
                slowed_heading.z - velocity.z,
            );

            let diff_len = diff.length();
            if diff_len > 0.0 {
                let c = (delta_seconds * friction * ACC_MULT / diff_len.max(1.0)).min(1.0);
                let acc = c * diff;
                velocity.x += acc.x;
                velocity.z += acc.z;
            }

            // Apply velocity with collision detection
            let (new_position, new_velocity, on_ground) = apply_velocity_with_collision(
                world,
                position,
                velocity,
                state.realm,
                aabb,
                delta_seconds,
            );

            return PhysicsStepResult {
                new_position,
                new_velocity,
                on_ground,
            };
        }
    }
}

/// Apply velocity to position with collision detection
fn apply_velocity_with_collision<W: BlockAccess>(
    world: &W,
    mut position: Vec3,
    mut velocity: Vec3,
    realm: Realm,
    aabb: Vec3,
    delta_seconds: f32,
) -> (Vec3, Vec3, bool) {
    let applied_velocity = velocity * delta_seconds;
    let mut on_ground = false;

    // X axis collision
    let xpos = if applied_velocity.x > 0. {
        aabb.x + position.x
    } else {
        position.x
    };
    let mut stopped = false;
    for x in extent(xpos, applied_velocity.x).into_iter().skip(1) {
        let pos_x = Vec3 {
            x: x as f32,
            y: position.y,
            z: position.z,
        };
        if blocks_perp_x(pos_x, realm, aabb).any(|pos| !world.get_block_safe(pos).is_traversable())
        {
            if applied_velocity.x > 0. {
                position.x = pos_x.x - aabb.x - 0.001;
            } else {
                position.x = pos_x.x + 1.001;
            }
            velocity.x = 0.;
            stopped = true;
            break;
        }
    }
    if !stopped {
        position.x += applied_velocity.x;
    }

    // Y axis collision
    let ypos = if applied_velocity.y > 0. {
        aabb.y + position.y
    } else {
        position.y
    };
    let mut stopped = false;
    for y in extent(ypos, applied_velocity.y).into_iter().skip(1) {
        let pos_y = Vec3 {
            x: position.x,
            y: y as f32,
            z: position.z,
        };
        if blocks_perp_y(pos_y, realm, aabb).any(|pos| !world.get_block_safe(pos).is_traversable())
        {
            if applied_velocity.y > 0. {
                position.y = pos_y.y - aabb.y - 0.001;
            } else {
                position.y = pos_y.y + 1.001;
                on_ground = true;
            }
            velocity.y = 0.;
            stopped = true;
            break;
        }
    }
    if !stopped {
        position.y += applied_velocity.y;
    }

    // Z axis collision
    let zpos = if applied_velocity.z > 0. {
        aabb.z + position.z
    } else {
        position.z
    };
    let mut stopped = false;
    for z in extent(zpos, applied_velocity.z).into_iter().skip(1) {
        let pos_z = Vec3 {
            x: position.x,
            y: position.y,
            z: z as f32,
        };
        if blocks_perp_z(pos_z, realm, aabb).any(|pos| !world.get_block_safe(pos).is_traversable())
        {
            if applied_velocity.z > 0. {
                position.z = pos_z.z - aabb.z - 0.001;
            } else {
                position.z = pos_z.z + 1.001;
            }
            velocity.z = 0.;
            stopped = true;
            break;
        }
    }
    if !stopped {
        position.z += applied_velocity.z;
    }

    (position, velocity, on_ground)
}

/// Convert transmittable actions to movement input
pub fn actions_to_movement_input(
    inputs: &bevy::platform::collections::HashSet<crate::messages::TransmittableAction>,
    camera_transform: &Transform,
) -> MovementInput {
    use crate::messages::TransmittableAction;

    let forward = camera_transform.forward().as_vec3();
    let right = camera_transform.right().as_vec3();

    let mut move_direction = Vec3::ZERO;
    let mut jump = false;
    let mut crouch = false;

    for action in inputs {
        match action {
            TransmittableAction::MoveForward => move_direction.z += 1.0,
            TransmittableAction::MoveBackward => move_direction.z -= 1.0,
            TransmittableAction::MoveRight => move_direction.x += 1.0,
            TransmittableAction::MoveLeft => move_direction.x -= 1.0,
            TransmittableAction::JumpOrFlyUp => jump = true,
            TransmittableAction::CrouchOrFlyDown => crouch = true,
            _ => {}
        }
    }

    MovementInput {
        move_direction,
        jump,
        crouch,
        camera_forward: forward,
        camera_right: right,
    }
}
