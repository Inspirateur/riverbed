use bevy::prelude::*;

use crate::items::HOTBAR_SLOTS;
const SLOT_SIZE: f32 = 80.;

#[derive(Component)]
pub struct HotbarNode;

pub fn setup_hotbar_display(
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
    .insert(HotbarNode)
    .with_children(|parent| {
        for i in 0..HOTBAR_SLOTS {
            let right_border = if i < HOTBAR_SLOTS - 1 { Val::Px(0.) } else { Val::Px(5.) };
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(SLOT_SIZE),
                    height: Val::Px(SLOT_SIZE),
                    border: UiRect::new(Val::Px(5.), right_border, Val::Px(5.), Val::Px(5.)),
                    ..Default::default()
                },
                border_color: BorderColor(Color::GRAY),
                ..Default::default()
            });
        }
    });
}

pub fn update_hotbar(hotbar: Query<&HotbarNode>) {

}