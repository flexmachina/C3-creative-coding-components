#import utils

// Vertex shader

struct VertexInput {
    @location(0)
    position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,

    @location(0)
    uv: vec3<f32>,
}

struct Camera {
    view_proj_inv: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(@builtin(vertex_index) id: u32,) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.uv = (camera.view_proj_inv * out.clip_position).xyz;
    return out;
}

// Fragment shader

@group(1) @binding(0)
var cube_texture: texture_cube<f32>;

@group(1) @binding(1)
var cube_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(cube_texture, cube_sampler, in.uv);
}
