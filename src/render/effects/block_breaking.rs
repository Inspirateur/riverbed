use bevy::prelude::*;
use crate::agents::{BreakingAction, TargetBlock};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct BreakingEffect(Entity);

pub fn add_break_animation(
    mut commands: Commands, 
    block_action_query: Query<(Entity, &TargetBlock, &BreakingAction), Without<BreakingEffect>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture_handle = asset_server.load("PixelPerfection/textures/blocks/destroy_stage_4.png");

    for (player, target_opt, _breaking) in block_action_query.iter() {
        let Some(target) = &target_opt.0 else {
            continue;
        };
        let quad_handle = meshes.add(Rectangle::new(1., 1.));
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let translation = <_ as Into<Vec3>>::into(target.pos) 
            + target.normal.max(Vec3::ZERO) 
            + target.normal*0.001 
            + (Vec3::ONE-target.normal.abs())*0.5;
        let effect = commands.spawn(PbrBundle {
            mesh: quad_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_translation(translation).looking_at(translation-target.normal, Vec3::Y),
            ..default()
        }).id();
        commands.entity(player).insert(BreakingEffect(effect));
    }
}

pub fn update_break_animation(block_action_query: Query<(Entity, &TargetBlock, &BreakingAction, &BreakingEffect)>) {
    for (player, target_block_opt, action, break_effect) in block_action_query.iter() {
    }
}

pub fn remove_break_animation(mut commands: Commands, block_action_query: Query<(Entity, &BreakingEffect), Without<BreakingAction>>) {
    for (player, breaking_effect) in block_action_query.iter() {
        commands.entity(breaking_effect.0).despawn();
        commands.entity(player).remove::<BreakingEffect>();
    }
}
