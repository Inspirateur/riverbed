use crate::bloc::Bloc;
use crate::load_cols::{ColLoadEvent, ColUnloadEvent};
use crate::player::Action;
use crate::pos::Pos;
use crate::world_data::WorldData;
use anyhow::Result;
use bevy::prelude::*;
use colorsys::Rgb;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashMap;
use std::str::FromStr;

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

pub fn on_col_load(
    mut commands: Commands,
    ev_load: EventReader<ColLoadEvent>,
    world: Res<WorldData>,
) {
}

pub fn on_col_unload(
    mut commands: Commands,
    ev_unload: EventReader<ColUnloadEvent>,
    world: Res<WorldData>,
) {
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
            .add_startup_system(setup)
            .add_system(update_cam)
            .add_system(on_col_load)
            .add_system(on_col_unload);
    }
}
