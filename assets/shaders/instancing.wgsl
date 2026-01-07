#import bevy_pbr::{
    pbr_types::{pbr_input_new},
    mesh_functions::{
        get_world_from_local, 
        mesh_position_local_to_clip, 
        mesh_position_local_to_world
    },
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}
};
// Seems wrong but the chunk size is actually 62 (optimal for greedy meshing with neighbor block padding)
const CHUNK_SIZE: i32 = 62;
const MASK_6BIT: u32 = 0x3Fu;
const MASK_7BIT: u32 = 0x7Fu;
const MASK_8BIT: u32 = 0xFFu;
const MASK_10BIT: u32 = 0x3FFu;
const MAX_7BIT: f32 = 127.0;
const MAX_8BIT: f32 = 255.0;
@group(3) @binding(0) var array_texture: texture_2d_array<f32>;
@group(3) @binding(1) var texture_sampler: sampler;
@group(3) @binding(2) var<storage, read> anim_offsets: array<u32>;
@group(3) @binding(3) var<storage, read> chunk_face_groups: array<vec4<i32>>;

struct VertexIn {
    @location(0) corner: vec3<f32>,   // unit quad corners
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) quad_data: vec3<u32>,// packed [xyzwh, packed face/layer/color, draw_id]
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) tex_index: u32,
    @location(4) face_light: vec4<f32>,
};

struct FaceBasis {
    u: vec3<f32>,
    v: vec3<f32>,
    n: vec3<f32>,
};

fn basis_from_face(face: u32) -> FaceBasis {
    // Face order from binary-greedy-meshing:
    // 0 Up(+Y), 1 Down(-Y), 2 Right(+X), 3 Left(-X), 4 Front(+Z), 5 Back(-Z)
    //
    // IMPORTANT: basis.u is the direction for QUAD WIDTH  (w)
    //            basis.v is the direction for QUAD HEIGHT (h)
    //
    // Chosen to be right-handed: cross(u, v) == n
    switch face {
        case 0u: { // Up (+Y): w -> +X, h -> +Z
            return FaceBasis(
                vec3( 1.0, 0.0, 0.0),  // +X (width)
                vec3( 0.0, 0.0,-1.0),  // -Z (height) (sign picked for RH)
                vec3( 0.0, 1.0, 0.0)   // +Y
            );
        }
        case 1u: { // Down (-Y): w -> -X, h -> +Z
            return FaceBasis(
                vec3(-1.0, 0.0, 0.0),  // -X (width)
                vec3( 0.0, 0.0,-1.0),  // -Z (height) (RH)
                vec3( 0.0,-1.0, 0.0)   // -Y
            );
        }
        case 2u: { // Right (+X): w -> -Y, h -> +Z
            return FaceBasis(
                vec3( 0.0,-1.0, 0.0),  // -Y (width)  (vertical trunks)
                vec3( 0.0, 0.0,-1.0),  // -Z (height) (RH)
                vec3( 1.0, 0.0, 0.0)   // +X
            );
        }
        case 3u: { // Left (-X): w -> -Y, h -> +Z
            return FaceBasis(
                vec3( 0.0,-1.0, 0.0),  // -Y (width)
                vec3( 0.0, 0.0, 1.0),  // +Z (height) (RH)
                vec3(-1.0, 0.0, 0.0)   // -X
            );
        }
        case 4u: { // Front (+Z): w -> +X, h -> -Y
            return FaceBasis(
                vec3(-1.0, 0.0, 0.0),  // -X (width)  (RH)
                vec3( 0.0,-1.0, 0.0),  // -Y (height) (vertical)
                vec3( 0.0, 0.0, 1.0)   // +Z
            );
        }
        default: { // Back (-Z): w -> -X, h -> -Y
            return FaceBasis(
                vec3(-1.0, 0.0, 0.0),  // -X (width)
                vec3( 0.0, 1.0, 0.0),  // +Y (height) (RH)
                vec3( 0.0, 0.0,-1.0)   // -Z
            );
        }
    }
}

@vertex
fn vertex(v: VertexIn) -> VertexOut {
    var out: VertexOut;

    let x  =  i32(v.quad_data[0] & MASK_6BIT);
    let y  = i32((v.quad_data[0] >> 6 ) & MASK_6BIT);
    let z  = i32((v.quad_data[0] >> 12) & MASK_6BIT);
    let w  = f32((v.quad_data[0] >> 18) & MASK_6BIT);
    let h  = f32((v.quad_data[0] >> 24) & MASK_6BIT);
    let layer_low = v.quad_data[0] >> 30;
    let layer  = ((v.quad_data[1] & MASK_10BIT) << 2) | layer_low;
    let col   =  v.quad_data[1] >> 10;

    // for now we use the manually set, per instance draw_id
    // but this will be replaced with proper draw id when wgpu supports it
    let draw_id = v.quad_data[2];
    let chunk_face_group = chunk_face_groups[draw_id];
    let face = u32(chunk_face_group.w);
    // Use stable unit quad coordinates (0..1) from UVs.
    let c = v.uv;

    let basis = basis_from_face(face);

    // Origin inside the chunk, then expand along face basis.
    let origin = vec3<f32>(
        f32(x + chunk_face_group.x*CHUNK_SIZE), 
        f32(y + chunk_face_group.y*CHUNK_SIZE), 
        f32(z + chunk_face_group.z*CHUNK_SIZE)
    );
    let local_pos = origin + basis.u * (c.x * w) + basis.v * (c.y * h);
    let local = vec4<f32>(local_pos, 1.0);

    let world_from_local = get_world_from_local(0);
    out.clip_position = mesh_position_local_to_clip(world_from_local, local);
    out.world_position = mesh_position_local_to_world(world_from_local, local);
    out.world_normal = normalize(basis.n);
    out.uv = vec2<f32>(c.x * w, c.y * h);
    // Unpack color 24 bits: r7 g8 b7
    let b = f32(col        & MASK_7BIT) / MAX_7BIT;
    let g = f32((col >> 7) & MASK_8BIT) / MAX_8BIT;
    let r = f32((col >>15) & MASK_7BIT) / MAX_7BIT;
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