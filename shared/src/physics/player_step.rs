use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::messages::TransmittableAction;
use crate::physics::{
    actions_to_movement_input, simulate_physics_step, MovementMode, PhysicsState, PhysicsStepResult,
};
use crate::world::block_access::BlockAccess;

/// Result of applying a single input frame to a player's physics state.
#[derive(Debug, Clone, Copy)]
pub struct PlayerStepOutput {
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub movement_mode: MovementMode,
}

/// Apply movement actions (including fly toggle) and simulate a physics step.
///
/// This is shared between client prediction/reconciliation and server authority
/// to keep movement behavior identical.
pub fn apply_player_input_step<W: BlockAccess>(
    world: &W,
    state: &PhysicsState,
    actions: &HashSet<TransmittableAction>,
    camera: &Transform,
    delta_seconds: f32,
) -> PlayerStepOutput {
    let mut movement_mode = state.movement_mode;
    let mut velocity = state.velocity;

    // Handle fly-mode toggle
    if actions.contains(&TransmittableAction::ToggleFlyMode) {
        movement_mode = match movement_mode {
            MovementMode::Walking => MovementMode::Flying,
            MovementMode::Flying => MovementMode::Walking,
        };

        // Reset vertical velocity when returning to walking to avoid ghost motion
        if movement_mode == MovementMode::Walking {
            velocity = Vec3::ZERO;
        }
    }

    let movement_input = actions_to_movement_input(actions, camera);

    let sim_state = PhysicsState {
        position: state.position,
        velocity,
        movement_mode,
        realm: state.realm,
        on_ground: state.on_ground,
    };

    let result: PhysicsStepResult =
        simulate_physics_step(world, &sim_state, &movement_input, delta_seconds);

    PlayerStepOutput {
        position: result.new_position,
        velocity: result.new_velocity,
        on_ground: result.on_ground,
        movement_mode,
    }
}
