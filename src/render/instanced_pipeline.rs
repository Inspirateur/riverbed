use crate::render::quad_data::*;
use crate::render::texture_array::BindGroupHandles;
use bevy::pbr::SetMeshViewBindingArrayBindGroup;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::render::storage::GpuShaderStorageBuffer;
use bevy::render::texture::GpuImage;
use bevy::{
    camera::visibility::NoFrustumCulling,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{SystemParamItem, lifetimeless::*},
    },
    mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout},
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        mesh::{RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::ExtractedView,
    },
};
const VOXEL_BIND_GROUP_INDEX: usize = 3;
/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/instancing.wgsl";

pub struct InstancedPipelinePlugin;

impl Plugin for InstancedPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_plugins(ExtractComponentPlugin::<InstanceQuads>::default());
        app.add_plugins(ExtractResourcePlugin::<BindGroupHandles>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(RenderStartup, init_custom_pipeline)
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSystems::QueueMeshes),
                    prepare_voxel_buffers.in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        InstanceQuads::default(),
        // NOTE: Baked in frustrum culling will only use the mesh's bounding box, which is wrong for instanced meshes.
        NoFrustumCulling,
    ));
}

impl ExtractComponent for InstanceQuads {
    type QueryData = &'static InstanceQuads;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceQuads>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    for (view, msaa) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr) | MeshPipelineKey::ATMOSPHERE;
        for (entity, main_entity) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom,
                // NOTE: This instanced voxel world is effectively a giant opaque surface,
                // but it's queued into Transparent3d to use a custom pipeline.
                // forcing -INF makes the world draw before other transparent items.
                distance: -f32::INFINITY,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

#[derive(Resource)]
struct VoxelBindGroup(BindGroup);

#[derive(Component)]
struct InstanceBuffer {
    quad_buffer: Buffer,
    indirect_buffer: Buffer,
    length: usize,
}

fn prepare_voxel_buffers(
    mut commands: Commands,
    query: Single<(Entity, &InstanceQuads)>,
    cam: Single<&Transform, With<Camera>>,
    handles: If<Res<BindGroupHandles>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    custom_pipeline: Res<CustomPipeline>,
    shader_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    // get the quads to send to the GPU
    let cam = cam.into_inner();
    let view_direction = cam.forward().normalize();
    let view_origin = cam.translation;
    let (entity, instance_data) = query.into_inner();
    let (
        instance_data, 
        indirect_buffer, 
        chunk_face_groups
    ) = instance_data.read().culled(view_direction, view_origin);

    if indirect_buffer.len() != chunk_face_groups.len() {
        println!(
            "voxel indirect draws ({}) != chunk_face_groups ({}) -> skipping frame to avoid OOB",
            indirect_buffer.len(),
            chunk_face_groups.len()
        );
        return;
    }
    // create the quad buffer and associated indirect buffer
    let quad_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("instance data buffer"),
        contents: bytemuck::cast_slice(instance_data.as_slice()),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    }); 
    let indirect_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("indirect draw buffer"),
        contents: bytemuck::cast_slice(indirect_buffer.iter().map(|(i, c)|
            DrawIndexedIndirectArgs {
                // The amount of vertices in Plane3d is 6
                index_count: 6,
                instance_count: *c as u32,
                first_index: 0,
                base_vertex: 0,
                first_instance: *i as u32,
            }
        ).collect::<Vec<_>>().as_slice()),
        usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
    });
    let chunk_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("chunk face groups buffer"),
        contents: bytemuck::cast_slice(chunk_face_groups.as_slice()),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });
    if chunk_buffer.size() == 0 {
        return;
    }
    // insert the quad and associated indirect buffer to the entity
    commands.entity(entity).insert(InstanceBuffer {
        quad_buffer,
        indirect_buffer,
        length: instance_data.len(),
    });
    // (re)create the voxel bind group because chunk buffer can change size each frame 
    let gpu_image = gpu_images.get(&handles.array_texture).unwrap();
    let anim = shader_buffers.get(&handles.anim_offsets).unwrap();

    let bind_group = render_device.create_bind_group(
        "voxel bind group",
        &pipeline_cache.get_bind_group_layout(&custom_pipeline.voxel_bind_group_layout),
        &BindGroupEntries::sequential(
            (
                &gpu_image.texture_view, 
                &gpu_image.sampler, 
                anim.buffer.as_entire_binding(),
                chunk_buffer.as_entire_binding(),
            )
        ),
    );
    commands.insert_resource(VoxelBindGroup(bind_group));
   
}


#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    pub voxel_bind_group_layout: BindGroupLayoutDescriptor,
}

fn init_custom_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_pipeline: Res<MeshPipeline>,
    adapter_info: Res<RenderAdapterInfo>,
) {
    println!(
        "wgpu adapter: {:?} / {:?} / {}",
        adapter_info.backend,
        adapter_info.device_type,
        adapter_info.name
    );

    let layout = BindGroupLayoutDescriptor::new(
        "voxel bind group layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            (
                binding_types::texture_2d_array(TextureSampleType::Float { filterable: true }),
                binding_types::sampler(SamplerBindingType::Filtering),
                binding_types::storage_buffer_read_only_sized(false, None),
                binding_types::storage_buffer_read_only_sized(false, None),
            ),
        ),
    );

    commands.insert_resource(CustomPipeline {
        shader: asset_server.load(SHADER_ASSET_PATH),
        mesh_pipeline: mesh_pipeline.clone(),
        voxel_bind_group_layout: layout,
    });
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<QuadData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![VertexAttribute {
                format: VertexFormat::Uint32x3,
                offset: 0,
                shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
            }],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        // bind group 3: texture array, sampler, animation offsets
        descriptor.layout.push(self.voxel_bind_group_layout.clone());
        debug_assert!(descriptor.layout.len() == VOXEL_BIND_GROUP_INDEX + 1);
        descriptor.primitive.cull_mode = Some(Face::Back);
        descriptor.primitive.front_face = FrontFace::Cw;
        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    SetVoxelBindGroup<3>,
    DrawMeshInstanced,
);

struct SetVoxelBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetVoxelBindGroup<I> {
    type Param = (
        Option<SRes<VoxelBindGroup>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        _query: Option<()>,
        voxel_bind_group_opt: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(voxel_bind_group) = voxel_bind_group_opt.0 else {
            return RenderCommandResult::Skip;
        };
        let bind_group = voxel_bind_group.into_inner();
        pass.set_bind_group(I, &bind_group.0, &[]);
        RenderCommandResult::Success
    }
}

struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: Option<&'w InstanceBuffer>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // A borrow check workaround.
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some(instance_buffer) = instance_buffer else {
            return RenderCommandResult::Skip;
        };
        if instance_buffer.length == 0 {
            return RenderCommandResult::Skip;
        }
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };
        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.quad_buffer.slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count: _,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), *index_format);
                let stride = size_of::<DrawIndexedIndirectArgs>() as u32;
                pass.multi_draw_indexed_indirect(
                    &instance_buffer.indirect_buffer, 
                    0, 
                    instance_buffer.indirect_buffer.size() as u32/stride
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
