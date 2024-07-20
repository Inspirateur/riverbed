use bevy::prelude::*;

pub struct CraftMenuPlugin;

impl Plugin for CraftMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_craft_menu)
            ;
    }
}

#[derive(Component)]
struct CraftMenu;

fn setup_craft_menu(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            display: Display::None,
            ..Default::default()
        },
        ..Default::default()
    }).insert(CraftMenu);
}

fn toggle_craft_menu() {
    
}