use bevy::prelude::*;
use shared::messages::ServerChatHistory;

#[derive(Message)]
pub struct ChatMessageEvent;

pub fn setup_chat_resources(app: &mut App) {
    app.insert_resource(ServerChatHistory { ..default() });
    app.add_message::<ChatMessageEvent>();
}
