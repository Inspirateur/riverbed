use std::ffi::OsStr;
use bevy::{asset::LoadedFolder, prelude::*, render::texture::TRANSPARENT_IMAGE_HANDLE};
use itertools::Itertools;
use crate::{agents::{BlockActionType, BlockLootAction, TargetBlock}, render::{BlockTexState, BlockTextureFolder}};

pub struct BlockBreakingEffectPlugin;

impl Plugin for BlockBreakingEffectPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(BreakStageSprites(vec![TRANSPARENT_IMAGE_HANDLE]))
            .add_systems(OnEnter(BlockTexState::Finished), load_break_stage_sprites)
            .add_systems(Update, add_break_animation)
            .add_systems(Update, update_break_animation)
            .add_systems(Update, remove_break_animation)
            ;
    }
}

#[derive(Resource)]
struct BreakStageSprites(Vec<Handle<Image>>);

#[derive(Component)]
#[component(storage = "SparseSet")]
struct BreakingEffect(Entity);

fn parse_break_stage_name(filename: &OsStr) -> Option<u32> {
    let filename = filename.to_str()?;
    let stage = filename.strip_prefix("destroy_stage_")?;
    stage.parse().ok()
}

fn load_break_stage_sprites(
    block_textures: Res<BlockTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut break_stages: ResMut<BreakStageSprites>,
) {
    let block_folder: &LoadedFolder = loaded_folders.get(&block_textures.0).unwrap();
    let mut breaking_sprites: Vec<(u32, Handle<Image>)> = Vec::new();
    for block_handle in block_folder.handles.iter() {
        let filename = block_handle.path().unwrap().path().file_stem().unwrap();
        let Some(break_stage) = parse_break_stage_name(filename) else {
            continue;
        };
        breaking_sprites.push((break_stage, block_handle.clone().try_typed().unwrap()));
    }
    breaking_sprites.sort_by(|(stage1, _), (stage2, _)| stage1.cmp(stage2));
    breaking_sprites.insert(0, (0, TRANSPARENT_IMAGE_HANDLE));
    break_stages.0 = breaking_sprites.into_iter().map(|(_, handle)| handle).collect_vec();
}

fn add_break_animation(
    mut commands: Commands, 
    block_action_query: Query<(Entity, &TargetBlock, &BlockLootAction), Without<BreakingEffect>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    break_stages: Res<BreakStageSprites>
) {
    let texture_handle = break_stages.0[0].clone_weak();

    for (player, target_opt, breaking) in block_action_query.iter() {
        if !matches!(breaking.action_type, BlockActionType::Breaking) {
            continue;
        } 
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

fn update_break_animation(
    block_action_query: Query<(&BlockLootAction, &BreakingEffect)>,
    break_stages: Res<BreakStageSprites>,
    mat_query: Query<&Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (action, break_effect) in block_action_query.iter() {
        let stage = (
            (break_stages.0.len() as f32*(1.-action.time_left.max(0.)/action.break_entry.hardness.unwrap_or(f32::INFINITY))) as usize
        ).min(break_stages.0.len());
        let Ok(mat_handle) = mat_query.get(break_effect.0) else {
            continue;
        };
        let Some(mat) = materials.get_mut(mat_handle) else {
            continue;
        };
        mat.base_color_texture = Some(break_stages.0[stage].clone());
    }
}

fn remove_break_animation(mut commands: Commands, block_action_query: Query<(Entity, &BreakingEffect, Option<&BlockLootAction>)>) {
    for (player, breaking_effect, opt_block_loot) in block_action_query.iter() {
        if opt_block_loot.is_some_and(|block_loot| matches!(block_loot.action_type, BlockActionType::Breaking)) {
            continue;
        }
        commands.entity(breaking_effect.0).despawn();
        commands.entity(player).remove::<BreakingEffect>();
    }
}
