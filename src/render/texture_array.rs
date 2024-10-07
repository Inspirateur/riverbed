use std::sync::Arc;
use bevy::{asset::LoadedFolder, pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline}, prelude::*, reflect::TypePath, render::{mesh::MeshVertexBufferLayoutRef, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat}}};
use dashmap::DashMap;
use crate::{Block, block::{Face, FaceSpecifier}, render::parse_block_tex_name};
use super::{mesh_chunks::ATTRIBUTE_VOXEL_DATA, BlockTexState, BlockTextureFolder};

#[derive(Resource)]
pub struct TextureMap(pub Arc<DashMap<(Block, FaceSpecifier), usize>>);

pub trait TextureMapTrait {
    fn get_texture_index(&self, block: Block, face: Face) -> usize;
}


impl TextureMapTrait for &DashMap<(Block, FaceSpecifier), usize> {
    // TODO: need to allow the user to create a json with "texture files links" such as:
    // grass_block_bottom.png -> dirt.png
    // furnace_bottom.png -> stone.png
    // etc ...
    fn get_texture_index(&self, block: Block, face: Face) -> usize {
        for specifier in face.specifiers() {
            if let Some(i) = self.get(&(block, *specifier)) {
                return *i
            }
        }
        0
    }
}

fn missing_tex(model: &Image) -> Image {
    let mut img = Image::new_fill(
        Extent3d {
            width: model.width(), height: model.width(), ..Default::default()
        }, TextureDimension::D2, &[130, 130, 130, 255], model.texture_descriptor.format, RenderAssetUsages::default()
    );
    let w = model.width() as usize;
    let pixels = w*w;
    let half_w = w/2;
    for i in 0..pixels {
        let (x, y) = ((i%w)/half_w, i/(w*half_w));
        if x != y {
            continue;
        }
        img.data[i*4] = 255;
        img.data[i*4+2] = 200;
    }
    img
}

fn build_tex_array(
    mut commands: Commands,
    block_textures: Res<BlockTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    texture_map: Res<TextureMap>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>>,
    // mut shader_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut texture_list: Vec<&Image> = Vec::new();
    let mut anim_offsets = vec![1];
    let mut index = 1;
    let loaded_folder: &LoadedFolder = loaded_folders.get(&block_textures.0).unwrap();
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
        let Some((block, face_specifier)) = parse_block_tex_name(filename) else {
            continue;
        };
        let frames = texture.height()/texture.width();
        texture_map.0.insert((block, face_specifier), index);
        texture_list.push(texture);
        for _ in 0..frames {
            anim_offsets.push(frames);
            index += 1;    
        }
    }
    let default = Image::new_fill(
        Extent3d { width: 2, height: 2, ..Default::default() }, 
        TextureDimension::D2, 
        &[100], 
        TextureFormat::Rgba8Unorm, 
        RenderAssetUsages::default()
    );
    let model = texture_list.get(0).cloned().unwrap_or(&default);
    let missing_tex = missing_tex(model);
    texture_list.insert(0, &missing_tex);
    let array_tex = Image::new(Extent3d {
            width: model.width(), 
            height: model.height(), 
            depth_or_array_layers: index as u32
        }, 
        TextureDimension::D2, 
        texture_list.into_iter().flat_map(|tex| tex.data.clone()).collect(), 
        model.texture_descriptor.format,
        RenderAssetUsages::default()
    );
    let handle = textures.add(array_tex);
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 1.,
            reflectance: 0.1,
            alpha_mode: AlphaMode::AlphaToCoverage,
            ..Default::default()
        },
        extension: ArrayTextureMaterial {
            array_texture: handle, anim_offsets
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
    #[storage(102, read_only)]
    anim_offsets: Vec<u32>,
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
            layout: &MeshVertexBufferLayoutRef,
            _key: MaterialExtensionKey<ArrayTextureMaterial>,
        ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
            let vertex_layout = layout.0.get_layout(&[ATTRIBUTE_VOXEL_DATA.at_shader_location(0)])?;
            descriptor.vertex.buffers = vec![vertex_layout];
            Ok(())
    }
}

pub struct TextureArrayPlugin;

impl Plugin for TextureArrayPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TextureMap(Arc::new(DashMap::new())))
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>::default())
            .add_systems(OnEnter(BlockTexState::Finished), build_tex_array)
            ;
    }
}
