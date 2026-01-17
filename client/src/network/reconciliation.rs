//! Server-authoritative reconciliation for client-side prediction.
//!
//! This module implements the client-side prediction and reconciliation system
//! for a server-authoritative networking model. The server is the single source
//! of truth (SSOT) for player positions.
//!
//! ## How it works:
//! 1. Client predicts movement locally for responsive gameplay
//! 2. Client sends inputs to server
//! 3. Server simulates authoritatively and broadcasts updates
//! 4. Client receives server state and reconciles:
//!    - If server state matches prediction, no correction needed
//!    - If mismatch, snap to server state and replay unacknowledged inputs

use bevy::prelude::*;
use shared::messages::ServerPlayerUpdate;
use shared::physics::{
    player_step::apply_player_input_step, MovementMode, PhysicsState,
};
use shared::world::realm::Realm;

use crate::agents::{PlayerControlled, Velocity, FreeFly, Walking, Speed};
use crate::network::CurrentPlayerProfile;
use shared::net::input_history::InputHistory;
use crate::world::ClientWorldMap;

/// Threshold for position correction. If the difference between client and server
/// position is less than this, we don't correct (to avoid jitter from floating point).
pub const POSITION_CORRECTION_THRESHOLD: f32 = 0.01;

/// Maximum allowed position error before we force a hard snap (teleport).
/// Below this threshold, we interpolate smoothly.
pub const HARD_SNAP_THRESHOLD: f32 = 5.0;

/// Interpolation factor for smooth corrections (0.0 = no correction, 1.0 = instant snap)
pub const CORRECTION_LERP_FACTOR: f32 = 0.3;

/// Plugin for client-side reconciliation
pub struct ReconciliationPlugin;

impl Plugin for ReconciliationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reconcile_player_state);
    }
}

/// System that reconciles the local player's state with server updates.
/// 
/// This is the core of client-side prediction reconciliation:
/// 1. When we receive a server update for our player, compare with prediction
/// 2. If there's a significant mismatch, correct our position
/// 3. Replay any inputs that the server hasn't acknowledged yet
pub fn reconcile_player_state(
    mut ev_update: MessageReader<ServerPlayerUpdate>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &Realm, Option<&FreeFly>),
        With<PlayerControlled>,
    >,
    mut commands: Commands,
    player_entity: Query<Entity, With<PlayerControlled>>,
    current_player: Res<CurrentPlayerProfile>,
    mut input_history: ResMut<InputHistory>,
    world: Res<ClientWorldMap>,
) {
    for event in ev_update.read() {
        // Only process updates for our own player
        if event.id != current_player.id {
            continue;
        }

        // Remove acknowledged inputs (server has processed these)
        let acked_count = input_history.ack_until(event.last_ack_time);

        if acked_count > 0 {
            debug!(
                "Acknowledged {} inputs (remaining: {})",
                acked_count,
                input_history.unacknowledged.len()
            );
        }

        let Ok((mut transform, mut velocity, realm, free_fly_opt)) = player_query.single_mut() else {
            warn!("No local player entity found for reconciliation");
            continue;
        };

        let Ok(entity) = player_entity.single() else {
            continue;
        };

        // Calculate position difference
        let position_diff = event.position - transform.translation;
        let position_error = position_diff.length();

        // Determine the final movement mode by replaying unacknowledged inputs.
        // This is necessary because the server state may not yet reflect toggle
        // actions that the client has sent but not been acknowledged.
        let final_movement_mode = if input_history.unacknowledged.is_empty() {
            // No unacked inputs, server state is authoritative
            event.movement_mode
        } else {
            // Replay unacked inputs to determine predicted movement mode
            let mut mode = event.movement_mode;
            for input in input_history.unacknowledged.iter() {
                if input.inputs.contains(&shared::messages::TransmittableAction::ToggleFlyMode) {
                    mode = match mode {
                        MovementMode::Walking => MovementMode::Flying,
                        MovementMode::Flying => MovementMode::Walking,
                    };
                }
            }
            mode
        };

        // Update ECS components to match the predicted movement mode (after replay)
        let predicted_is_flying = final_movement_mode == MovementMode::Flying;
        let client_is_flying = free_fly_opt.is_some();
        
        if predicted_is_flying != client_is_flying {
            if predicted_is_flying {
                commands.entity(entity).remove::<Walking>().insert(FreeFly);
                commands.entity(entity).insert(Speed(shared::FLY_SPEED));
            } else {
                commands.entity(entity).remove::<FreeFly>().insert(Walking);
                commands.entity(entity).insert(Speed(shared::WALK_SPEED));
            }
            info!("Movement mode corrected: flying={}", predicted_is_flying);
        }

        if position_error < POSITION_CORRECTION_THRESHOLD {
            // Position is close enough, no correction needed
            // Just update velocity to match server (helps with prediction)
            velocity.0 = event.velocity;
            continue;
        }

        if position_error > HARD_SNAP_THRESHOLD {
            // Large error - hard snap to server position
            warn!(
                "Large position error ({:.2}m), hard snapping to server position",
                position_error
            );
            transform.translation = event.position;
            velocity.0 = event.velocity;
        } else {
            // Small error - apply correction and replay unacknowledged inputs
            debug!(
                "Position error: {:.3}m, replaying {} unacked inputs",
                position_error,
                input_history.unacknowledged.len()
            );

            // Start from server's authoritative state
            let mut corrected_state = PhysicsState {
                position: event.position,
                velocity: event.velocity,
                movement_mode: event.movement_mode,
                realm: *realm,
                on_ground: false,
            };

            // Replay all unacknowledged inputs
            for input in input_history.unacknowledged.iter() {
                let delta_seconds = input.delta_ms as f32 / 1000.0;
                let step = apply_player_input_step(
                    &*world,
                    &corrected_state,
                    &input.inputs,
                    &input.camera,
                    delta_seconds,
                );

                corrected_state.position = step.position;
                corrected_state.velocity = step.velocity;
                corrected_state.movement_mode = step.movement_mode;
                corrected_state.on_ground = step.on_ground;
            }

            // Apply the corrected state with smooth interpolation
            // This reduces visual jitter while still correcting errors
            transform.translation = transform.translation.lerp(corrected_state.position, CORRECTION_LERP_FACTOR);
            velocity.0 = corrected_state.velocity;
        }
    }
}
