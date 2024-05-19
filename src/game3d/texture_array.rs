use std::{ffi::OsStr, str::FromStr, sync::Arc};
use bevy::{asset::LoadedFolder, pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline}, prelude::*, reflect::TypePath, render::{render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension}}};
use dashmap::DashMap;
use itertools::Itertools;
use crate::blocks::{Block, Face};

use super::render3d::ATTRIBUTE_VOXEL_DATA;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FaceSpecifier {
    Specific(Face),
    Side,
    All
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum TexState {
    #[default]
    Setup,
    Finished
}

#[derive(Resource, Default)]
struct TextureFolder(Handle<LoadedFolder>);

#[derive(Resource)]
pub struct TextureMap(pub Arc<DashMap<(Block, FaceSpecifier), usize>>);

pub trait TextureMapTrait {
    fn get_texture_index(&self, block: Block, face: Face) -> Option<usize>;
}


impl TextureMapTrait for DashMap<(Block, FaceSpecifier), usize> {
    // TODO: need to allow the user to create a json with "texture files links" such as:
    // grass_block_bottom.png -> dirt.png
    // furnace_bottom.png -> stone.png
    // etc ...
    fn get_texture_index(&self, block: Block, face: Face) -> Option<usize> {
        if let Some(i) = self.get(&(block, FaceSpecifier::Specific(face))) {
            return Some(*i);
        }
        if matches!(face, Face::Front | Face::Back | Face::Left | Face::Right) {
            if let Some(i) = self.get(&(block, FaceSpecifier::Side)) {
                return Some(*i);
            }
        }
        if face == Face::Down {
            if let Some(i) = self.get(&(block, FaceSpecifier::Specific(Face::Up))) {
                return Some(*i);
            }
        }
        if let Some(res) = self.get(&(block, FaceSpecifier::All)) {
            Some(*res)
        } else {
            None
        }
    }
}

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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn parse_block_name(blockname: &str) -> Option<Block> {
    Block::from_str(&blockname.split("_").map(capitalize).join("")).ok()
}

fn parse_tex_name(filename: &OsStr) -> Option<(Block, FaceSpecifier)> {
    let filename = filename.to_str()?;
    let Some((block, face)) = filename.rsplit_once("_") else {
        return Some((parse_block_name(filename)?, FaceSpecifier::All));
    };
    match face {
        "side" => Some((parse_block_name(block)?, FaceSpecifier::Side)),
        "bottom" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Down))),
        "top" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Up))),
        "front" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Front))),
        "side1" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Left))),
        "side2" => Some((parse_block_name(block)?, FaceSpecifier::Specific(Face::Right))),
        _ => Some((parse_block_name(filename)?, FaceSpecifier::All))
    }
}

fn setup(
    mut commands: Commands,
    textures_handles: Res<TextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    texture_map: Res<TextureMap>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>>,
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
        let Some((block, face_specifier)) = parse_tex_name(filename) else {
            continue;
        };
        texture_map.0.insert((block, face_specifier), texture_list.len());
        texture_list.push(texture);
    }
    if texture_list.len() == 0 {
        return;
    }
    let model = texture_list[0];
    println!("{:?}", model.texture_descriptor.format);
    let array_tex = Image::new(Extent3d {
            width: model.width(), 
            height: model.height(), 
            depth_or_array_layers: texture_list.len() as u32
        }, 
        TextureDimension::D2, 
        texture_list.into_iter().flat_map(|tex| tex.data.clone()).collect(), 
        model.texture_descriptor.format,
        RenderAssetUsages::default()
    );
    let handle = textures.add(array_tex);
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 0.9,
            reflectance: 0.2,
            ..Default::default()
        },
        extension: ArrayTextureMaterial {
            array_texture: handle,
        }
    });
    commands.insert_resource(BlockTextureArray(handle));
}


#[derive(Resource)]
pub struct BlockTextureArray(pub Handle<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>);

#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct ArrayTextureMaterial {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    array_texture: Handle<Image>,
}

impl MaterialExtension for ArrayTextureMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn specialize(
            _pipeline: &MaterialExtensionPipeline,
            descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
            layout: &bevy::render::mesh::MeshVertexBufferLayout,
            _key: MaterialExtensionKey<ArrayTextureMaterial>,
        ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
            let vertex_layout = layout.get_layout(&[ATTRIBUTE_VOXEL_DATA.at_shader_location(0)])?;
            descriptor.vertex.buffers = vec![vertex_layout];
            Ok(())
    }
}

pub struct TextureArrayPlugin;

impl Plugin for TextureArrayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TexState>()
            .insert_resource(TextureMap(Arc::new(DashMap::new())))
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>::default())
            .add_systems(OnEnter(TexState::Setup), load_textures)
            .add_systems(Update, check_textures.run_if(in_state(TexState::Setup)))
            .add_systems(OnEnter(TexState::Finished), setup)
            ;
    }
}
