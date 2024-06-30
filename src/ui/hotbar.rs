use std::collections::HashMap;
use bevy::prelude::*;
use crate::{agents::PlayerControlled, items::{Hotbar, Item, HOTBAR_SLOTS}};
const SLOT_SIZE: f32 = 60.;

#[derive(Resource)]
struct UITextureMap(HashMap<Item, Handle<Image>>);

#[derive(Component)]
struct HotbarUI { pub selected: usize }

fn setup_hotbar_display(
    mut commands: Commands
) {
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(SLOT_SIZE),
            width: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(HotbarUI { selected: 0})
    .with_children(|parent| {
        for i in 0..HOTBAR_SLOTS {
            let right_border = if i < HOTBAR_SLOTS - 1 { Val::Px(0.) } else { Val::Px(4.) };
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(SLOT_SIZE),
                    height: Val::Px(SLOT_SIZE),
                    border: UiRect::new(Val::Px(4.), right_border, Val::Px(4.), Val::Px(4.)),
                    ..Default::default()
                },
                border_color: BorderColor(Color::BLACK),
                ..Default::default()
            }).with_children(|parent| {
                parent.spawn(ImageBundle {
                    ..Default::default()
                }).insert(Text2dBundle {
                    ..Default::default()
                });
            });
        }
    });
}

fn update_hotbar(
    hotbar_ui_query: Query<(&Children, &HotbarUI)>, 
    mut slot_query: Query<(&mut BackgroundColor, &Children)>, 
    mut img_query: Query<&mut Handle<Image>>, 
    mut text_query: Query<&mut Text>,
    tex_map: Res<UITextureMap>,
    hotbar_query: Query<&Hotbar, (With<PlayerControlled>, Changed<Hotbar>)>,
) {
    let Ok(hotbar) = hotbar_query.get_single() else {
        // The hotbar hasn't change since last time
        return;
    };
    let Ok((ui_slots, hotbar_ui)) = hotbar_ui_query.get_single() else {
        // The hotbar ui is not present (should this panic ?)
        return;
    };
    let mut i = 0;
    for ui_slots_e in ui_slots {
        let (mut ui_slot, slot) = slot_query.get_mut(*ui_slots_e).unwrap();
        ui_slot.0 = if hotbar_ui.selected == i {
            Color::rgba(1., 1., 1., 0.6)
        } else {
            Color::rgba(1., 1., 1., 0.2)
        };
        // TODO: display the correct image and number for the item stack
        i += 1;
    }
}

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_hotbar_display)
            .add_systems(Update, update_hotbar)
            ;
    }
}