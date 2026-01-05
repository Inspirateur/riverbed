#import bevy_pbr::{
    pbr_types::{pbr_input_new},
    mesh_functions::{
        get_world_from_local, 
        mesh_position_local_to_clip, 
        mesh_position_local_to_world
    },
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}
};
const CHUNK_SIZE: i32 = 62;
@group(3) @binding(0) var array_texture: texture_2d_array<f32>;
@group(3) @binding(1) var texture_sampler: sampler;
@group(3) @binding(2) var<storage, read> anim_offsets: array<u32>;
@group(3) @binding(3) var<storage, read> chunk_face_groups: array<vec4<i32>>;

struct VertexIn {
    @location(0) corner: vec3<f32>,   // unit quad corners
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) quad_data: vec2<u32>,// packed [xyzwh, packed face/layer/color]
    @builtin(instance_index) instance: u32,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) tex_index: u32,
    @location(4) face_light: vec4<f32>,
};

fn normal_from_face(face: u32) -> vec3<f32> {
    switch face {
        case 0u: { return vec3( 1.0, 0.0, 0.0); }
        case 1u: { return vec3(-1.0, 0.0, 0.0); }
        case 2u: { return vec3( 0.0, 1.0, 0.0); }
        case 3u: { return vec3( 0.0,-1.0, 0.0); }
        case 4u: { return vec3( 0.0, 0.0, 1.0); }
        default: { return vec3( 0.0, 0.0,-1.0); }
    }
}

@vertex
fn vertex(v: VertexIn) -> VertexOut {
    var out: VertexOut;

    let x  = f32( v.quad_data[0] & 0x3Fu);
    let y  = f32((v.quad_data[0] >> 6 ) & 0x3Fu);
    let z  = f32((v.quad_data[0] >> 12) & 0x3Fu);
    let w  = f32((v.quad_data[0] >> 18) & 0x3Fu);
    let h  = f32((v.quad_data[0] >> 24) & 0x3Fu);

    let face  = (v.quad_data[1]        & 0x7u);
    let layer = (v.quad_data[1] >> 3)  & 0xFFFu;
    let col   =  v.quad_data[1] >> 15;

    // Map unit corner to face axes.
    // NOTE: `Plane3d` vertices lie in the XZ plane (Y is constant), so using
    // `corner.y` here would collapse the quad to a line. The mesh UVs provide
    // stable unit-quad coordinates (0..1, 0..1) regardless of plane orientation.
    let c = v.uv;
    var lx = c.x * w;
    var ly = c.y * h;
    var lz = 0.0;

    // Very simple face mapping (adjust to match your mesher)
    // XY faces: +/-Z faces rotate
    if (face == 4u) { // +Z
        let tmp = lx; lx = 0.0; lz = tmp;
    } else if (face == 5u) { // -Z
        let tmp = lx; lx = 0.0; lz = -tmp;
    } else if (face == 0u || face == 1u) { // +/-X -> use YZ plane
        let tmp = lx; lx = 0.0; lz = tmp; // customize
    }

    let chunk_face_group = chunk_face_groups[v.instance];
    let chunk_offset = vec4<f32>(
        vec3<f32>(chunk_face_group.xyz) * f32(CHUNK_SIZE),
        0.0
    );
    let local = vec4(x + lx, y + ly, z + lz, 1.0) + chunk_offset;
    let world_from_local = get_world_from_local(0);
    out.clip_position = mesh_position_local_to_clip(world_from_local, local);
    out.world_position = mesh_position_local_to_world(world_from_local, local);
    out.world_normal = normalize(normal_from_face(face));
    out.uv = c;
    // Unpack color 17 bits: r6 g6 b5
    let b = f32(col        & 0x1Fu) / 31.0;
    let g = f32((col >> 5) & 0x3Fu) / 63.0;
    let r = f32((col >>11) & 0x3Fu) / 63.0;
    out.tex_index = layer;
    // will be colored light later
    out.face_light = vec4(1.0);

    return out;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    var pbr_input = pbr_input_new();
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = in.world_normal;
    pbr_input.material.base_color = in.face_light 
        * textureSample(array_texture, texture_sampler, in.uv, in.tex_index);
    //    textureSampleBias(array_texture, texture_sampler, in.uv, in.tex_index + (u32(globals.time*2.0) % anim_offsets[in.texture_layer]), view.mip_bias);

    // pbr_input.material.base_color = fns::alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // apply lighting
    var color = apply_pbr_lighting(pbr_input);

    // color = main_pass_post_lighting_processing(pbr_input, out.color);

    return color;
}