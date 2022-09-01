use crate::bloc::Bloc;
use crate::load_cols::{ColLoadEvent, ColUnloadEvent};
use crate::world_data::WorldData;
use anyhow::Result;
use bevy::prelude::*;
use colorsys::Rgb;
use std::collections::HashMap;
use std::str::FromStr;

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
            .add_system(on_col_load)
            .add_system(on_col_unload);
    }
}
