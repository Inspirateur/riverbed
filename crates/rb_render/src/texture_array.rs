use super::{
    BlockTexState, BlockTextureFolder, mesh_logic::ATTRIBUTE_VOXEL_DATA, parse_block_tex_name,
};
use bevy::{
    asset::{LoadedFolder, RenderAssetUsages},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
    prelude::*,
    reflect::TypePath,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, TextureDataOrder, TextureDimension, TextureFormat,
        },
        storage::ShaderBuffer,
    },
    shader::ShaderRef,
};
use hashbrown::HashMap;
use rb_block::{Block, Face, FaceSpecifier};

pub struct TextureArrayPlugin;

impl Plugin for TextureArrayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TextureMap(HashMap::new()))
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>,
            >::default())
            .add_systems(OnEnter(BlockTexState::Loaded), build_tex_array);
    }
}

#[derive(Resource)]
pub struct TextureMap(pub HashMap<(Block, FaceSpecifier), usize>);

pub trait TextureMapTrait {
    fn get_texture_index(&self, block: Block, face: Face) -> usize;
}

impl TextureMapTrait for &HashMap<(Block, FaceSpecifier), usize> {
    // TODO: need to allow the user to create a json with "texture files links" such as:
    // grass_block_bottom.png -> dirt.png
    // furnace_bottom.png -> stone.png
    // etc ...
    fn get_texture_index(&self, block: Block, face: Face) -> usize {
        for specifier in face.specifiers() {
            if let Some(i) = self.get(&(block, *specifier)) {
                return *i;
            }
        }
        0
    }
}

fn missing_tex(model: &Image) -> Image {
    let mut img = Image::new_fill(
        Extent3d {
            width: model.width(),
            height: model.width(),
            ..Default::default()
        },
        TextureDimension::D2,
        &[130, 130, 130, 255],
        model.texture_descriptor.format,
        RenderAssetUsages::default(),
    );
    let w = model.width();
    let pixels = w * w;
    let half_w = w / 2;
    for i in 0..pixels {
        let (x, y) = ((i % w) / half_w, i / (w * half_w));
        if x != y {
            continue;
        }
        img.set_color_at(x, y, Color::srgb(1., 0.5, 0.5)).unwrap();
    }
    img
}

fn downsample_image_by_half(image: &Image) -> Image {
    let width = image.width();
    let height = image.height();

    if width == 1 && height == 1 {
        return image.clone();
    }

    let new_width = (width / 2).max(1);
    let new_height = (height / 2).max(1);
    let source = image.data.as_ref().expect("Image has no pixel data");
    let mut pixels = vec![0u8; (new_width * new_height * 4) as usize];

    for y in 0..new_height {
        for x in 0..new_width {
            let mut sum = [0u32; 4];
            let mut count = 0u32;

            let start_x = x * width / new_width;
            let end_x = (x + 1) * width / new_width;
            let start_y = y * height / new_height;
            let end_y = (y + 1) * height / new_height;

            for source_y in start_y..end_y {
                for source_x in start_x..end_x {
                    let source_index = ((source_y * width + source_x) * 4) as usize;

                    for channel in 0..4 {
                        sum[channel] += source[source_index + channel] as u32;
                    }

                    count += 1;
                }
            }

            let destination_index = ((y * new_width + x) * 4) as usize;

            for channel in 0..4 {
                pixels[destination_index + channel] = (sum[channel] / count) as u8;
            }
        }
    }

    let mut downsampled = image.clone();
    downsampled.texture_descriptor.size.width = new_width;
    downsampled.texture_descriptor.size.height = new_height;
    downsampled.data = Some(pixels);

    downsampled
}

fn resize_image(image: &Image, size: u32) -> Image {
    if image.width() == size && image.height() == size {
        return image.clone();
    }

    let source = image.data.as_ref().expect("Image has no pixel data");
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let source_x = x * image.width() / size;
            let source_y = y * image.height() / size;
            let source_index = ((source_y * image.width() + source_x) * 4) as usize;
            let destination_index = ((y * size + x) * 4) as usize;
            pixels[destination_index..destination_index + 4]
                .copy_from_slice(&source[source_index..source_index + 4]);
        }
    }

    let mut resized = image.clone();
    resized.texture_descriptor.size.width = size;
    resized.texture_descriptor.size.height = size;
    resized.data = Some(pixels);
    resized
}

// Takes a full resolution image texture as a base and generates a chain of downsampled versions
// Returned as one full image with mip level set
fn build_mip_chain(base: &Image, target_size: u32) -> (Vec<Vec<u8>>, u32) {
    let frame_size = base.width();
    assert_eq!(
        base.height() % frame_size,
        0,
        "Texture height must be a multiple of its width"
    );

    let source = base.data.as_ref().expect("Image has no pixel data");
    let bytes_per_frame = (frame_size * frame_size * 4) as usize;
    let mut frames: Vec<Image> = source
        .chunks_exact(bytes_per_frame)
        .map(|pixels| {
            let mut frame = base.clone();
            frame.texture_descriptor.size.height = frame_size;
            frame.data = Some(pixels.to_vec());
            frame
        })
        .map(|frame| resize_image(&frame, target_size))
        .collect();
    let mip_level_count = target_size.ilog2() + 1;
    let mut levels = Vec::with_capacity(mip_level_count as usize);

    for level in 0..mip_level_count {
        levels.push(
            frames
                .iter()
                .flat_map(|frame| frame.data.as_ref().unwrap().iter().copied())
                .collect(),
        );
        if level + 1 < mip_level_count {
            frames = frames.iter().map(downsample_image_by_half).collect();
        }
    }

    (levels, mip_level_count)
}

fn build_tex_array(
    mut commands: Commands,
    block_textures: Res<BlockTextureFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut texture_map: ResMut<TextureMap>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>>,
    mut next_state: ResMut<NextState<BlockTexState>>,
    mut shader_buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let mut texture_list: Vec<&Image> = Vec::new();
    let mut anim_offsets = vec![1];
    let mut index = 1;
    let loaded_folder: &LoadedFolder = loaded_folders.get(&block_textures.0).unwrap();
    let mut water_layer = None;
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
        let frames = texture.height() / texture.width();

        texture_map.0.insert((block, face_specifier), index);
        texture_list.push(texture);

        if block == Block::SeaBlock {
            water_layer = Some(index);
        }
        for _ in 0..frames {
            anim_offsets.push(frames);
            index += 1;
        }
    }
    let default = Image::new_fill(
        Extent3d {
            width: 2,
            height: 2,
            ..Default::default()
        },
        TextureDimension::D2,
        &[100, 100, 25, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    let model = texture_list
        .iter()
        .copied()
        .max_by_key(|texture| texture.width())
        .unwrap_or(&default);
    let texture_size = model.width();
    let missing_tex = missing_tex(model);
    texture_list.insert(0, &missing_tex);

    let mip_chains: Vec<_> = texture_list
        .iter()
        .map(|texture| build_mip_chain(texture, texture_size))
        .collect();
    let mip_level_count = mip_chains[0].1;
    let data = (0..mip_level_count as usize)
        .flat_map(|level| {
            mip_chains
                .iter()
                .flat_map(move |(levels, _)| levels[level].iter().copied())
        })
        .collect();

    let mut array_tex = Image::new_uninit(
        Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: index as u32,
        },
        TextureDimension::D2,
        model.texture_descriptor.format,
        RenderAssetUsages::default(),
    );
    array_tex.data_order = TextureDataOrder::MipMajor;
    array_tex.data = Some(data);
    array_tex.texture_descriptor.mip_level_count = mip_level_count;
    let handle = textures.add(array_tex);
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 1.,
            reflectance: 0.1,
            alpha_mode: AlphaMode::AlphaToCoverage,
            ..Default::default()
        },
        extension: ArrayTextureMaterial {
            array_texture: handle,
            anim_offsets: shader_buffers.add(ShaderBuffer::from(anim_offsets)),
            water_layer: water_layer.unwrap() as u32,
        },
    });
    commands.insert_resource(BlockTextureArray(handle));
    next_state.set(BlockTexState::Mapped);
}

#[derive(Resource)]
pub struct BlockTextureArray(pub Handle<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>);

#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct ArrayTextureMaterial {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    array_texture: Handle<Image>,
    #[storage(102, read_only)]
    anim_offsets: Handle<ShaderBuffer>,
    #[uniform(103)]
    water_layer: u32,
}

impl MaterialExtension for ArrayTextureMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    // Used for the depth, normal, and motion-vector prepasses as well as
    // shadow map generation (Bevy uses the prepass shader for shadow depth).
    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/prepass_chunk.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/prepass_chunk.wgsl".into()
    }

    fn enable_prepass() -> bool {
        true
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<ArrayTextureMaterial>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[ATTRIBUTE_VOXEL_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
