use crate::bloc::Bloc;
use crate::chunk::Chunk;
use crate::load_cols::{ColLoadEvent, ColUnloadEvent};
use crate::player::Action;
use crate::pos::Pos;
use crate::realm::Realm;
use crate::world_data::WorldData;
use anyhow::Result;
use bevy::prelude::*;
use colorsys::Rgb;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashMap;
use std::str::FromStr;
const IM_SIZE: f32 = 64.;

pub fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

pub fn update_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Pos, (With<ActionState<Action>>, Changed<Pos>)>,
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_pos) = player_query.get_single() {
            cam_pos.translation = player_pos.coord;
        }
    }
}

pub fn render(col: &[Option<Chunk>]) -> Image {
    todo!()
}

pub fn on_col_load(
    mut commands: Commands,
    mut ev_load: EventReader<ColLoadEvent>,
    world: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
    mut col_ents: ResMut<HashMap<(Realm, i32, i32), Entity>>,
) {
    for ColLoadEvent((realm, x, z)) in ev_load.iter() {
        let ent = commands.spawn_bundle(SpriteBundle {
            texture: images.add(render(world.chunks.col(*realm, *x, *z))),
            transform: Transform::from_translation(Vec3::new(*x as f32, 0., *z as f32) * IM_SIZE),
            ..default()
        });
        col_ents.insert((*realm, *x, *z), ent.id());
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut col_ents: ResMut<HashMap<(Realm, i32, i32), Entity>>,
) {
    for col_ev in ev_unload.iter() {
        if let Some(ent) = col_ents.remove(&col_ev.0) {
            commands.entity(ent).despawn();
        }
    }
}

pub struct SoilColor(HashMap<Bloc, Rgb>);

impl SoilColor {
    pub fn from_csv(path: &str) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut data = HashMap::new();
        for record in reader.records() {
            let record = record?;
            let color = Rgb::from_hex_str(&record[1])?;
            data.insert(Bloc::from_str(&record[0]).unwrap(), color);
        }
        Ok(SoilColor(data))
    }
}
pub struct Draw2d;

impl Plugin for Draw2d {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoilColor::from_csv("assets/data/soils_color.csv").unwrap())
            .insert_resource(HashMap::<(Realm, i32, i32), Entity>::new())
            .add_startup_system(setup)
            .add_system(update_cam)
            .add_system(on_col_load)
            .add_system(on_col_unload);
    }
}
