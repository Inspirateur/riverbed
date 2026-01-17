pub mod buffered_client;
mod chat;
mod cleanup;
pub mod extensions;
mod inputs;
pub mod models;
pub mod save;
mod setup;
mod world;

pub use chat::*;
pub use cleanup::*;
pub use extensions::SendGameMessageExtension;
pub use inputs::*;
pub use setup::*;

use bevy::prelude::*;

use crate::network::buffered_client::{CurrentFrameInputs, PlayerTickInputsBuffer, SyncTime};

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<CurrentPlayerProfile>()
            .init_resource::<PlayerTickInputsBuffer>()
            .init_resource::<CurrentFrameInputs>()
            .init_resource::<SyncTime>()
            .init_resource::<UnacknowledgedInputs>()
            .init_resource::<SelectedWorld>()
            .init_resource::<ClientTime>()
            .init_resource::<WorldSeed>();
        
        // Setup base netcode plugins (RenetClientPlugin, NetcodeClientPlugin)
        add_base_netcode(app);

        // Startup systems - run once at launch
        app.add_systems(
            Startup,
            (launch_local_server_system, init_server_connection).chain(),
        );

        // Update systems - run every frame
        app.add_systems(
            Update,
            (
                establish_authenticated_connection_to_server,
                network_failure_handler,
            ),
        );

        // Fixed update systems - run at fixed timestep
        app.add_systems(FixedPreUpdate, poll_network_messages);
        app.add_systems(FixedUpdate, upload_player_inputs_system);
    }
}
