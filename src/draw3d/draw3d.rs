use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy::window::CursorGrabMode;
use leafwing_input_manager::prelude::*;
use crate::blocs::{Pos, Blocs, ChunkPos, CHUNK_S1, Y_CHUNKS, ChunkChanges, Pos2D};
use crate::GameState;
use crate::gen::{LoadedCols, ColUnloadEvent};
use crate::agents::{AABB, TargetBloc, Dir};
use crate::draw3d::render3d::Meshable;
use crate::sky::SkyPlugin;
use crate::draw3d::texture_array::{TextureMap, TextureArrayPlugin};
const CAMERA_PAN_RATE: f32 = 0.1;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 150., 10.)
            .looking_at(Vec3 {x: 0., y: 150., z: 0.}, Vec3::Y),
        ..Default::default()
    })
    .insert(InputManagerBundle::<CameraMovement> {
        input_map: InputMap::default()
            // This will capture the total continuous value, for direct use.
            // Note that you can also use discrete gesture-like motion, via the `MouseMotionDirection` enum.
            .insert(DualAxis::mouse_motion(), CameraMovement::Pan)
            .build(),
        ..default()
    }).insert(FpsCam::default());
    let mut window = windows.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

pub fn translate_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<(&Pos, &AABB), (With<ActionState<Dir>>, Changed<Pos>)>
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok((player_pos, aabb)) = player_query.get_single() {
            cam_pos.translation.x = player_pos.x + aabb.0.x/2.;
            cam_pos.translation.y = player_pos.y + aabb.0.y;
            cam_pos.translation.z = player_pos.z + aabb.0.z/2.;
        }
    }
}

fn pan_camera(mut query: Query<(&ActionState<CameraMovement>, &mut FpsCam)>, time: Res<Time>) {
    let (action_state, mut fpscam) = query.single_mut();
    let camera_pan_vector = action_state.axis_pair(CameraMovement::Pan).unwrap();
    let c = time.delta_seconds() * CAMERA_PAN_RATE;
    fpscam.yaw -= c*camera_pan_vector.x();
    fpscam.pitch -= c*camera_pan_vector.y();
    fpscam.pitch = fpscam.pitch.clamp(-1.4, 1.4);
}

fn apply_fps_cam(mut query: Query<(&mut Transform, &FpsCam)>) {
    let (mut transform, fpscam) = query.single_mut();
    transform.rotation = Quat::from_axis_angle(Vec3::Y, fpscam.yaw) * Quat::from_axis_angle(Vec3::X, fpscam.pitch);
}

fn target_bloc(
    mut player: Query<(&mut TargetBloc, &Pos<f32>), With<ActionState<Dir>>>, 
    player_cam: Query<&Transform, With<FpsCam>>,
    world: Res<Blocs>
) {
    let (mut target_bloc, player_pos) = player.single_mut();
    let transform = player_cam.single();
    target_bloc.0 = world.raycast(
        player_pos.realm, 
        transform.translation, 
        transform.forward(), 
        12.
    );
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut chunk_ents: ResMut<ChunkEntities>,
) {
    for col_ev in ev_unload.iter() {
        for i in 0..Y_CHUNKS {
            if let Some(ent) = chunk_ents.0.remove(&ChunkPos {
                x: col_ev.0.x,
                y: i as i32,
                z: col_ev.0.z,
                realm: col_ev.0.realm
            }) {
                commands.entity(ent).despawn();
            }
        }
    }
}

pub fn process_bloc_changes(
    loaded_cols: Res<LoadedCols>,
    mut commands: Commands,
    mut blocs: ResMut<Blocs>, 
    mesh_query: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_ents: ResMut<ChunkEntities>,
    texture_map: Res<TextureMap>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    if let Some((chunk, chunk_change)) = blocs.changes.pop() {
        if !loaded_cols.has_player(chunk.into()) { return; }
        match chunk_change {
            ChunkChanges::Created => {
                let ent = commands.spawn(PbrBundle {
                    mesh: meshes.add(blocs.fast_mesh(chunk, &texture_map)),
                    material: materials.add(Color::rgb(
                        if chunk.x % 2 == 0 { 0.8 } else { 0.4 }, 
                        if chunk.y % 2 == 0 { 0.8 } else { 0.4 }, 
                        if chunk.z % 2 == 0 { 0.8 } else { 0.4 }
                    ).into()),
                    transform: Transform::from_translation(
                        Vec3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32) * CHUNK_S1 as f32 - Vec3::new(1., 1., 1.),
                    ),
                    ..Default::default()
                }).insert(NoFrustumCulling).id();
                // this should not happen
                assert!(!chunk_ents.0.contains_key(&chunk));
                chunk_ents.0.insert(chunk, ent);
            },
            ChunkChanges::Edited(changes) => {
                if let Some(ent) = chunk_ents.0.get(&chunk) {
                    if let Ok(handle) = mesh_query.get_component::<Handle<Mesh>>(*ent) {
                        if let Some(mesh) = meshes.get_mut(&handle) {
                            blocs.process_changes(chunk, changes, mesh, &texture_map);
                        }
                    } else {
                        // the entity is not instanciated yet, we put it back
                        blocs.changes.insert(chunk, ChunkChanges::Edited(changes));
                    }
                }
            }
        }
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect)]
enum CameraMovement {
    Pan,
}


#[derive(Resource)]
pub struct ChunkEntities(pub HashMap::<ChunkPos, Entity>);

impl ChunkEntities {
    pub fn new() -> Self {
        ChunkEntities(HashMap::new())
    }
}

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SkyPlugin)
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .add_systems(Startup, setup)
            .add_systems(Update, translate_cam)
            .add_systems(Update, pan_camera.run_if(in_state(GameState::Game)))
            .add_systems(Update, target_bloc)
            .add_systems(Update, apply_fps_cam)
            .add_systems(Update, on_col_unload)
            .add_systems(Update, process_bloc_changes)
            ;
    }
}
