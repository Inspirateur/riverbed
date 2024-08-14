use bevy::{prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE};
use crate::{agents::{BlockActionType, BlockLootAction, PlayerControlled}, items::{Hotbar, Stack}, render::{CameraSpawn, FpsCam}};
use super::{hotbar::UiTextureMap, SelectedHotbarSlot};

pub struct InHandPlugin;

impl Plugin for InHandPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, in_hand_setup.after(CameraSpawn))
            .add_systems(Update, on_hotbar_change)
            .add_systems(Update, on_selected_slot_change)
            .add_systems(Update, animate)
            ;
    }
}

#[derive(Component)]
struct InHandMaterial;

fn in_hand_setup(
    mut commands: Commands, 
    cam_query: Query<Entity, With<FpsCam>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO: there seem to be a 1 frame delay in the position update of the in hand item, try to fix it
    // The player must be instanciated at this stage
    let Ok(cam) = cam_query.get_single() else  {
        println!("couldn't get the camera");
        return;
    };

    let material_handle = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let quad_handle = meshes.add(Rectangle::new(0.1, 0.1));

    commands.entity(cam)
        .with_children(|c| {
            c.spawn(PbrBundle {
                mesh: quad_handle,
                material: material_handle,
                transform: Transform::from_xyz(0.12, -0.08, -0.2).looking_at(Vec3::new(2., 0., -1.).normalize(), Vec3::new(0., 1., -0.4).normalize()),
                ..Default::default()
            }).insert(InHandMaterial);
        });
}

fn on_hotbar_change(
    hotbar_query: Query<&Hotbar, (With<PlayerControlled>, Changed<Hotbar>)>,
    in_hand_query: Query<&Handle<StandardMaterial>, With<InHandMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected_slot: Res<SelectedHotbarSlot>,
    tex_map: Res<UiTextureMap>,
) {
    let Ok(hotbar) = hotbar_query.get_single() else {
        return;
    };
    let Ok(in_hand) = in_hand_query.get_single() else {
        return;
    };
    let Some(in_hand_material) = materials.get_mut(in_hand) else {
        return;
    };
    let stack = &hotbar.0.0[selected_slot.0];
    let mut handle = TRANSPARENT_IMAGE_HANDLE;
    if let Stack::Some(item, _) = stack {
        if let Some(tex_handle) = tex_map.0.get(item) {
            handle = tex_handle.clone();
        }
    }
    in_hand_material.base_color_texture = Some(handle);
}

fn on_selected_slot_change(
    hotbar_query: Query<&Hotbar, With<PlayerControlled>>,
    in_hand_query: Query<&Handle<StandardMaterial>, With<InHandMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected_slot: Res<SelectedHotbarSlot>,
    tex_map: Res<UiTextureMap>,
) {
    if !selected_slot.is_changed() {
        return;
    }
    let Ok(hotbar) = hotbar_query.get_single() else {
        return;
    };
    let Ok(in_hand) = in_hand_query.get_single() else {
        return;
    };
    let Some(in_hand_material) = materials.get_mut(in_hand) else {
        return;
    };
    let stack = &hotbar.0.0[selected_slot.0];
    let mut handle = TRANSPARENT_IMAGE_HANDLE;
    if let Stack::Some(item, _) = stack {
        if let Some(tex_handle) = tex_map.0.get(item) {
            handle = tex_handle.clone();
        }
    }
    in_hand_material.base_color_texture = Some(handle);
}

fn animate(
    mut in_hand_query: Query<&mut Transform, With<InHandMaterial>>,
    loot_action_query: Query<Option<&BlockLootAction>, With<PlayerControlled>>
) {
    let Ok(loot_action_opt) = loot_action_query.get_single() else {
        return;
    };
    let Ok(mut transform) = in_hand_query.get_single_mut() else {
        return;
    };
    if let Some(loot_action) = loot_action_opt {
        *transform = match loot_action.action_type {
            BlockActionType::Breaking => transform.looking_at(
                Vec3::new(2., 0., -1.).normalize(), 
                Vec3::new(0., 1., -0.4+(loot_action.time_left*25.).sin()/4.).normalize()
            ),
            BlockActionType::Harvesting => transform.looking_at(
                Vec3::new(2., 0., -1.).normalize(), 
                Vec3::new(0., 1., -0.4+(loot_action.time_left*20.).sin()/6.).normalize()
            ),
        };
    } else {
        *transform = transform.looking_at(
            Vec3::new(2., 0., -1.).normalize(), 
            Vec3::new(0., 1., -0.4).normalize()
        )
    }
}