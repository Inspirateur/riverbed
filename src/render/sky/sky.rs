use std::{f32::consts::PI, time::Duration};
use bevy::{pbr::Atmosphere, prelude::*};
use crate::render::camera::{CameraSpawn, FpsCam};
const DAY_LENGTH_MINUTES: f32 = 0.2;
const C: f32 = DAY_LENGTH_MINUTES*120.*PI;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app   
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1000.0,
                ..Default::default()
            })
            .insert_resource(CycleTimer(Timer::new(
                 // Update our atmosphere every 500ms
                Duration::from_millis(500),
                TimerMode::Repeating,
            )))
            .add_systems(Startup, spawn_sun.after(CameraSpawn))
            .add_systems(Update, daylight_cycle)
            ;
    }
}

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);


#[derive(Component)]
struct Sun;

fn spawn_sun(mut commands: Commands, cam_query: Query<Entity, With<FpsCam>>) {
    let cam = cam_query.single().unwrap();
    commands.entity(cam).insert(Atmosphere::default());
    commands.spawn((Sun, DirectionalLight {
        // TODO: this crashes, maybe it will be fixed by following https://github.com/bevyengine/bevy/blob/main/assets/shaders/extended_material_bindless.wgsl
        // shadows_enabled: true,
        ..Default::default()
    }));
}

fn daylight_cycle(
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.is_finished() {
        // let t = 0.6 + time.elapsed_seconds_wrapped() / C;
        // TODO: make night time prettier with a skybox, freeze the sun in the meantime
        let t = 0.6f32;

        if let Some((mut light_trans, mut directional)) = query.single_mut().unwrap().into() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 50_000.0;
        }
    }
}
