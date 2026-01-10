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

fn normal_from_face(face: u32) -> vec3<f32> {
    // Face order from binary-greedy-meshing:
    // 0 Up(+Y), 1 Down(-Y), 2 Right(+X), 3 Left(-X), 4 Front(+Z), 5 Back(-Z)
    switch face {
        case 0u: { return vec3( 0.0, 1.0, 0.0); }
        case 1u: { return vec3( 0.0,-1.0, 0.0); }
        case 2u: { return vec3( 1.0, 0.0, 0.0); }
        case 3u: { return vec3(-1.0, 0.0, 0.0); }
        case 4u: { return vec3( 0.0, 0.0, 1.0); }
        default: { return vec3( 0.0, 0.0,-1.0); } // 5u Back
    }
}

fn uv_from_face(face: u32, uv: vec2<f32>, w: f32, h: f32) -> vec2<f32> {
    switch face {
        case 2u: { // Right (+X): 
            return vec2((1 - uv.y) * h, (1 - uv.x) * w);
        }
        case 3u: { // Left (-X)
            return vec2(uv.y * h, uv.x * w);
        }
        case 4u: { // Front (+Z):
            return vec2((1 - uv.x) * w, uv.y * h);
        }
        case 5u: { // Back (-Z): flip U to fix mirrored texture
            return vec2(uv.x * w, (1 - uv.y) * h);
        }
        default: {
            return vec2(uv.x * w, uv.y * h);
        }
    }
}

// Returns the local-space offset to add to `origin` for this face.
// `uv` is expected to be the unit quad coordinates (0..1).
fn vertex_from_face(face: u32, uv: vec2<f32>, w: f32, h: f32) -> vec3<f32> {
    // Convert unit quad coords into "block-space" extents.
    let u = uv.x * w;
    let v = uv.y * h;

    // Map (u,v) into XYZ depending on face.
    switch face {
        case 0u: { // Up (+Y):   w -> +X, h -> -Z
            return vec3( u, 0.0, h-v);
        }
        case 1u: { // Down (-Y): w -> -X, h -> -Z
            return vec3(-u, 0.0, h-v);
        }
        case 2u: { // Right (+X): w -> -Y, h -> -Z
            return vec3(0.0, u-w, v);
        }
        case 3u: { // Left (-X):  w -> -Y, h -> +Z
            return vec3(0.0, w-u,  v);
        }
        case 4u: { // Front (+Z): w -> -X, h -> -Y
            return vec3(-u, h-v, 0.0);
        }
        default: { // Back (-Z):  w -> +X, h -> -Y
            return vec3(w - u, v, 0.0);
        }
    }
}

// crude face shading to enhance readability of the terrain
fn light_from_face(face: u32) -> vec4<f32> {
    switch face {
        case 0u: { return vec4<f32>(1.0, 1.0, 1.0, 1.0); } // Up
        case 1u: { return vec4<f32>(0.5, 0.5, 0.5, 1.0); } // Down
        case 2u: { return vec4<f32>(0.6, 0.6, 0.6, 1.0); } // Right
        case 3u: { return vec4<f32>(0.6, 0.6, 0.6, 1.0); } // Left
        case 4u: { return vec4<f32>(0.7, 0.7, 0.7, 1.0); } // Front
        default: { return vec4<f32>(0.7, 0.7, 0.7, 1.0); } // Back
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

    // Origin inside the chunk, then expand along face basis.
    let origin = vec3<f32>(
        f32(x + chunk_face_group.x*CHUNK_SIZE), 
        f32(y + chunk_face_group.y*CHUNK_SIZE), 
        f32(z + chunk_face_group.z*CHUNK_SIZE)
    );
    let local_pos = origin + vertex_from_face(face, v.uv, w, h);
    let local = vec4<f32>(local_pos, 1.0);

    // IMPORTANT: don't index Bevy's mesh-instance transform buffer here.
    let world_from_local = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
    out.clip_position = mesh_position_local_to_clip(world_from_local, local);
    out.world_position = mesh_position_local_to_world(world_from_local, local);
    out.world_normal = normal_from_face(face);
    out.uv = uv_from_face(face, v.uv, w, h);
    // Unpack color 24 bits: r7 g8 b7
    let b = f32(col        & MASK_7BIT) / MAX_7BIT;
    let g = f32((col >> 7) & MASK_8BIT) / MAX_8BIT;
    let r = f32((col >>15) & MASK_7BIT) / MAX_7BIT;
    out.tex_index = layer;
    // will be colored light later
    out.face_light = vec4(r, g, b, 1.0) * light_from_face(face);

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