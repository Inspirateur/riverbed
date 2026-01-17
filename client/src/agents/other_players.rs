//! Handles rendering and updating other players in the game world.
//!
//! This module manages:
//! - Spawning visual entities for other players when they join
//! - Updating their positions based on server updates
//! - Removing them when they disconnect

use bevy::prelude::*;
use shared::messages::{PlayerId, PlayerSpawnEvent, PlayerUpdateEvent};

use crate::network::{CurrentPlayerProfile, UnacknowledgedInputs};

/// Marker component for other players (not the local player)
#[derive(Component)]
pub struct OtherPlayer {
    pub id: PlayerId,
    pub name: String,
}

/// Plugin for handling other players
pub struct OtherPlayersPlugin;

impl Plugin for OtherPlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_other_players, update_other_players));
    }
}

/// System to spawn entities for other players when they join
fn spawn_other_players(
    mut commands: Commands,
    mut ev_spawn: MessageReader<PlayerSpawnEvent>,
    current_player: Res<CurrentPlayerProfile>,
    existing_players: Query<&OtherPlayer>,
) {
    for event in ev_spawn.read() {
        // Skip if this is our own player
        if event.id == current_player.id {
            debug!("Ignoring spawn event for local player {}", event.id);
            continue;
        }

        // Skip if player already exists
        if existing_players.iter().any(|p| p.id == event.id) {
            debug!("Player {} already spawned", event.id);
            continue;
        }

        info!("Spawning other player: {} ({})", event.name, event.id);

        // Spawn a simple cube to represent the other player
        // TODO: Replace with proper player model/mesh
        commands.spawn((
            Transform::from_translation(event.data.position),
            Visibility::default(),
            OtherPlayer {
                id: event.id,
                name: event.name.clone(),
            },
        ));
    }
}

/// System to update other players' positions from server updates
fn update_other_players(
    mut ev_update: MessageReader<PlayerUpdateEvent>,
    mut other_players: Query<(&OtherPlayer, &mut Transform)>,
    current_player: Res<CurrentPlayerProfile>,
    mut unack_inputs: ResMut<UnacknowledgedInputs>,
) {
    for event in ev_update.read() {
        // Handle our own player update (for server reconciliation)
        if event.id == current_player.id {
            // Remove acknowledged inputs
            unack_inputs.0.retain(|input| input.time_ms > event.last_ack_time);
            
            // TODO: Implement proper client-side prediction reconciliation
            // For now, we trust the server position
            continue;
        }

        // Update other player's position
        for (player, mut transform) in other_players.iter_mut() {
            if player.id == event.id {
                transform.translation = event.position;
                transform.rotation = event.orientation;
                debug!(
                    "Updated player {} position to {:?}",
                    player.name, event.position
                );
            }
        }
    }
}
