use std::f32::consts::PI;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_mod_billboard::prelude::*;

use crate::game3d::camera::{CameraSpawn, FpsCam};
const TIME_FACTOR: f32 = 20.;

#[derive(Resource)]
struct Hour(f32);

#[derive(Component)]
struct Sun;

fn spawn_sun(mut commands: Commands, asset_server: Res<AssetServer>, cam_query: Query<Entity, With<FpsCam>>) {
    let angle = Vec3::new(1., 1., 0.1).normalize();
    let cam = cam_query.get_single().unwrap();
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: angle,
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder::default()
        .into(),
        ..default()
    }).insert(Sun);
    let sun = commands.spawn(BillboardTextBundle {
        transform: Transform::from_translation(Vec3::new(0., 100., 0.)),
        text: Text::from_sections([
            TextSection {
                value: "SUN".to_string(),
                style: TextStyle {
                    font_size: 60.0,
                    font: asset_server.load("fonts/RobotoMono-Light.ttf"),
                    color: Color::ORANGE,
                }
            }
        ]).with_alignment(TextAlignment::Center),
        ..default()
    })
    .insert(Sun).id();
    // commands.entity(cam).add_child(sun);
}

fn update_hour(mut hour: ResMut<Hour>, time: Res<Time>) {
    hour.0 = (hour.0 + time.delta_seconds()*TIME_FACTOR/3600.) % 24.;
}

fn rotate_sun(query: Query<&mut Sun>, hour: Res<Hour>) {

}

fn update_sunlight(query: Query<(&mut DirectionalLight, &mut Transform, &Sun)>) {

}

pub struct SkyPlugin;


impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(BillboardPlugin)
            .insert_resource(Hour(0.))
            .add_systems(Startup, spawn_sun.after(CameraSpawn))
            .add_systems(Update, update_hour)
            .add_systems(Update, rotate_sun)
            .add_systems(Update, update_sunlight)
            ;
    }
}