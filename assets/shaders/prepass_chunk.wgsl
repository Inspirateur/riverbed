// Prepass shader for voxel chunks (Bevy 0.18).
// Handles depth, normal, motion vector, and deferred prepasses.
// Also used as the shadow vertex shader by Bevy's material system.

#import bevy_pbr::{
    prepass_bindings,
    mesh_functions::{get_world_from_local, mesh_position_local_to_world},
    mesh_view_bindings::view,
    prepass_io::FragmentOutput,
    view_transformations::position_world_to_clip,
}

#ifdef MOTION_VECTOR_PREPASS
#import bevy_pbr::mesh_functions::get_previous_world_from_local
#endif

#ifdef DEFERRED_PREPASS
#import bevy_pbr::rgb9e5
#endif

// Bit masks matching the voxel vertex encoding in mesh_logic.rs / chunk.wgsl
const MASK3: u32 = 7u;
const MASK6: u32 = 63u;

// Decode face normal from a 3-bit ID (same encoding as the main chunk shader)
fn normal_from_id(id: u32) -> vec3<f32> {
    switch id {
        case 0u { return vec3(0.0, 1.0, 0.0); }   // +Y (top)
        case 1u { return vec3(0.0, -1.0, 0.0); }  // -Y (bottom)
        case 2u { return vec3(1.0, 0.0, 0.0); }   // +X (right)
        case 3u { return vec3(-1.0, 0.0, 0.0); }  // -X (left)
        case 4u { return vec3(0.0, 0.0, 1.0); }   // +Z (forward)
        case 5u { return vec3(0.0, 0.0, -1.0); }  // -Z (back)
        default { return vec3(0.0); }
    }
}

// Custom vertex input matching ATTRIBUTE_VOXEL_DATA (two u32s per vertex)
struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) voxel_data: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // World position is needed for motion vectors and deferred
    @location(0) world_position: vec4<f32>,
    // World normal is only needed for normal prepass and deferred prepass
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    @location(1) world_normal: vec3<f32>,
#endif
    // Previous world position is only needed for motion vector prepass
#ifdef MOTION_VECTOR_PREPASS
    @location(2) previous_world_position: vec4<f32>,
#endif
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Decode vertex position from the packed voxel_data.x
    // Bits  0-5:  x (6 bits)
    // Bits  6-11: y (6 bits)
    // Bits 12-17: z (6 bits)
    let vertex_info = vertex.voxel_data.x;
    let x = f32(vertex_info & MASK6);
    let y = f32((vertex_info >> 6u) & MASK6);
    let z = f32((vertex_info >> 12u) & MASK6);
    let local_position = vec4(x, y, z, 1.0);

    let world_from_local = get_world_from_local(vertex.instance_index);
    out.world_position = mesh_position_local_to_world(world_from_local, local_position);
    out.position = position_world_to_clip(out.world_position.xyz);

#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    // Decode face normal ID from voxel_data.y bits 0-2
    let quad_info = vertex.voxel_data.y;
    out.world_normal = normal_from_id(quad_info & MASK3);
#endif

#ifdef MOTION_VECTOR_PREPASS
    // Use the previous frame's model matrix to compute where this vertex was last frame.
    // For static chunks this equals world_position; motion comes only from camera changes.
    out.previous_world_position = mesh_position_local_to_world(
        get_previous_world_from_local(vertex.instance_index),
        local_position,
    );
#endif

    return out;
}

// When a fragment output is needed (normal / motion-vector / deferred prepass),
// Bevy defines PREPASS_FRAGMENT and FragmentOutput is non-empty.
#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

#ifdef NORMAL_PREPASS
    // Pack world-space normal into [0, 1] range for the normal buffer
    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
#endif

#ifdef MOTION_VECTOR_PREPASS
    // Project current and previous world positions into NDC, then compute
    // the per-pixel motion offset. Scaled by vec2(0.5, -0.5) so that a
    // full-screen motion maps to the [-1, 1] UV range with correct Y flip.
    let clip_curr = view.unjittered_clip_from_world * in.world_position;
    let clip_prev = prepass_bindings::previous_view_uniforms.clip_from_world * in.previous_world_position;
    let ndc_curr = clip_curr.xy / clip_curr.w;
    let ndc_prev = clip_prev.xy / clip_prev.w;
    out.motion_vector = (ndc_curr - ndc_prev) * vec2(0.5, -0.5);
#endif

#ifdef DEFERRED_PREPASS
    // Write a placeholder into the deferred G-buffer.
    // A full deferred implementation would encode base_color, roughness, etc.
    out.deferred = vec4(0u, bevy_pbr::rgb9e5::vec3_to_rgb9e5_(vec3(1.0, 0.0, 1.0)), 0u, 0u);
    out.deferred_lighting_pass_id = 1u;
#endif

    return out;
}
#else
// Depth-only / shadow pass: Bevy still compiles a fragment stage from our shader
// (because we provided a custom prepass_fragment_shader), but needs an entry point.
// No color outputs are written here; depth is handled implicitly via the position builtin.
@fragment
fn fragment(in: VertexOutput) {}
#endif // PREPASS_FRAGMENT
