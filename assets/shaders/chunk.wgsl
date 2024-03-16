#import bevy_pbr::{
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND, PbrInput, pbr_input_new},
    pbr_functions as fns,
    mesh_functions::{get_model_matrix, mesh_position_local_to_clip, mesh_position_local_to_world},
}
#import bevy_core_pipeline::tonemapping::tone_mapping

@group(2) @binding(0) var texture_pack: texture_2d_array<f32>;
@group(2) @binding(1) var texture_sampler: sampler;

const MASK2: u32 = 3;
const MASK3: u32 = 7;
const MASK4: u32 = 15;
const MASK6: u32 = 63;
const MASK9: u32 = 511;
const MASK16: u32 = 65535;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) voxel_data: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) texture_layer: u32,
    @location(5) face_light: f32,
};


fn normal_from_id(id: u32) -> vec3<f32> {
    var n: vec3<f32>;
    switch id {
        case 0u {
            n = vec3(-1.0, 0.0, 0.0);
        }
        case 1u {
            n = vec3(0.0, -1.0, 0.0);
        }
        case 2u {
            n = vec3(0.0, 0.0, -1.0);
        }
        case 3u {
            n = vec3(1.0, 0.0, 0.0);
        }
        case 4u {
            n = vec3(0.0, 1.0, 0.0);
        }
        case 5u {
            n = vec3(0.0, 0.0, 1.0);
        }
        default {
            n = vec3(0.0);
        }
    }
    return n;
}

fn light_from_id(id: u32) -> f32 {
    switch id {
        case 4u {
            return 1.0; // top
        }
        case 0u, 2u, 3u, 5u {
            return 0.7; // sides
        }
        case 1u {
            return 0.3; // bottom
        }
        default {
            return 0.0;
        }
    }
}

fn color_from_id(id: u32) -> vec4<f32> {
    var r = f32(id & MASK3)/7.0;
    var g = f32((id >> 3) & MASK3)/7.0;
    var b = f32((id >> 6) & MASK3)/7.0;
    return vec4(r, g, b, 1.0);
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var first = vertex.voxel_data.x;
    var x = f32(first & MASK6);
    var y = f32((first >> 6) & MASK6);
    var z = f32((first >> 12) & MASK6);
    var position = vec4(x, y, z, 1.0);
    var n_id = (first >> 18) & MASK3;
    var normal = normal_from_id(n_id);
    // var l_id = (first >> 21) & MASK2;
    var face_light = light_from_id(n_id);
    var c_id = (first >> 23) & MASK9;
    var face_color = color_from_id(c_id);
    
    var second = vertex.voxel_data.y;
    var u = f32(second & MASK6);
    var v = f32((second >> 6) & MASK6);
    var light = f32((second >> 12) & MASK4) / f32(MASK4);
    var texture_layer = second >> 16;

    var out: VertexOutput;
    out.position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        position,
    );
    out.world_position = mesh_position_local_to_world(
        get_model_matrix(vertex.instance_index),
        position,
    );
    out.world_normal = normal;
    out.uv = vec2(u, v);
    out.color = face_color;
    out.texture_layer = texture_layer;
    out.face_light = face_light;
    return out;
}

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
    // the material members
    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;
    pbr_input.material.perceptual_roughness = 1.0;
    pbr_input.material.reflectance = 0.2;
    pbr_input.material.base_color = textureSampleBias(texture_pack, texture_sampler, mesh.uv, mesh.texture_layer, view.mip_bias);
    pbr_input.material.base_color = pbr_input.material.base_color * mesh.color * mesh.face_light;

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = fns::apply_normal_mapping(
        pbr_input.material.flags,
        mesh.world_normal,
        double_sided,
        is_front,
#ifdef VERTEX_TANGENTS
#ifdef STANDARD_MATERIAL_NORMAL_MAP
        mesh.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
        mesh.uv,
#endif // VERTEX_UVS
        view.mip_bias,
    );
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);

    return tone_mapping(fns::apply_pbr_lighting(pbr_input), view.color_grading);
}