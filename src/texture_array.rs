use std::collections::HashMap;

use bevy::{prelude::*, reflect::{TypeUuid, TypePath}, render::render_resource::{ShaderRef, AsBindGroup}};
use ourcraft::{Bloc, Face};

#[derive(Resource)]
pub struct TextureMap(pub HashMap<(Bloc, Face), usize>);

pub fn create_tex_arr(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    todo!()
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid, TypePath)]
#[uuid = "9c5a0ddf-1eaf-41b4-9832-ed736fd26af3"]
pub struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    array_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
}

pub struct TextureArrayPlugin;

impl Plugin for TextureArrayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TextureMap(HashMap::new()))
            .add_plugins(MaterialPlugin::<ArrayTextureMaterial>::default())
            //.add_systems(Startup, create_tex_arr)
            ;
    }
}
