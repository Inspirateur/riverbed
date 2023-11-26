use std::collections::HashMap;
use bevy::{prelude::*, reflect::{TypeUuid, TypePath}, render::render_resource::{ShaderRef, AsBindGroup, Extent3d, TextureDimension}, asset::LoadedFolder};
use crate::{blocs::{Bloc, Face}, GameState};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum TexState {
    #[default]
    Setup,
    Finished
}

#[derive(Resource, Default)]
struct TextureFolder(Handle<LoadedFolder>);

#[derive(Resource)]
pub struct TextureMap(pub HashMap<(Bloc, Face), usize>);

fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a folder
    commands.insert_resource(TextureFolder(asset_server.load_folder("PixelPerfection/textures/blocks")));
}

fn check_textures(
    mut next_state: ResMut<NextState<TexState>>,
    texture_folder: ResMut<TextureFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&texture_folder.0) {
            next_state.set(TexState::Finished);
        }
    }
}

fn setup(
    textures_handles: Res<TextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut texture_map: ResMut<TextureMap>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    // Build a `TextureAtlas` using the individual sprites
    let mut texture_list: Vec<&Image> = Vec::new();
    let loaded_folder: &LoadedFolder = loaded_folders.get(&textures_handles.0).unwrap();
    for handle in loaded_folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };
        let filename = handle.path().unwrap().path().file_stem().unwrap();
        println!(
            "loaded texture {:?} {}x{}", 
            filename,
            texture.width(),
            texture.height()
        );
        texture_list.push(texture);
    }
    if texture_list.len() == 0 {
        return;
    }
    let model = texture_list[0];
    let array_tex = Image::new(Extent3d {
            width: model.width(), 
            height: model.height(), 
            depth_or_array_layers: texture_list.len() as u32
        }, 
        TextureDimension::D2, 
        texture_list.into_iter().flat_map(|tex| tex.data.clone()).collect(), 
        model.texture_descriptor.format
    );
    let handle = textures.add(array_tex);
    materials.add(ArrayTextureMaterial {
        array_texture: handle,
    });
}

#[derive(Asset, AsBindGroup, Debug, Clone, TypeUuid, TypePath)]
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
        app.add_state::<TexState>()
            .insert_resource(TextureMap(HashMap::new()))
            .add_plugins(MaterialPlugin::<ArrayTextureMaterial>::default())
            .add_systems(OnEnter(TexState::Setup), load_textures)
            .add_systems(Update, check_textures.run_if(in_state(TexState::Setup)))
            .add_systems(OnEnter(TexState::Finished), setup)
            ;
    }
}
