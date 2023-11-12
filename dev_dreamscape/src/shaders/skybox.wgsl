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

struct Data {
    proj_mat_inv: mat4x4<f32>,
    view_mat: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> data: Data;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position =  vec4<f32>(in.position, 1.0);

    var pos_unprojected = data.proj_mat_inv * out.clip_position;
    // Couldn't pass a 3x3 matrix in the uniform so transforming it into 3x3 here.
    // Also just using the raw 4x4 view matrix does not work because of its position component, apparently.
    var view_mat_inv = transpose(mat3x3<f32>(data.view_mat.x.xyz, data.view_mat.y.xyz, data.view_mat.z.xyz));
    out.uv = view_mat_inv * pos_unprojected.xyz;

    return out;
}

// Fragment shader

@group(1) @binding(0)
var cubeTexture: texture_cube<f32>;

@group(1) @binding(1)
var cubeSampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(cubeTexture, cubeSampler, in.uv);
}
