#import utils

// Vertex shader

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(1)
var<uniform> lights: array<Light, #MAX_LIGHTS>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_view_position: vec3<f32>,

    @location(3) tangent_light_position0: vec3<f32>,
    @location(4) tangent_light_position1: vec3<f32>,
    @location(5) tangent_light_position2: vec3<f32>,
    @location(6) tangent_light_position3: vec3<f32>,
    @location(7) tangent_light_position4: vec3<f32>,
    @location(8) tangent_light_position5: vec3<f32>,
    @location(9) tangent_light_position6: vec3<f32>,
    @location(10) tangent_light_position7: vec3<f32>,
    @location(11) tangent_light_position8: vec3<f32>,
    @location(12) tangent_light_position9: vec3<f32>,
    @location(13) tangent_light_position10: vec3<f32>,
    @location(14) tangent_light_position11: vec3<f32>,
    @location(15) tangent_light_position12: vec3<f32>,
    @location(16) tangent_light_position13: vec3<f32>,
    @location(17) tangent_light_position14: vec3<f32>,
    @location(18) tangent_light_position15: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    // Construct the tangent matrix
    let world_normal = normalize(normal_matrix * model.normal);
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;

    out.tangent_light_position0 = tangent_matrix * lights[0].position;
    out.tangent_light_position1 = tangent_matrix * lights[1].position;
    out.tangent_light_position2 = tangent_matrix * lights[2].position;
    out.tangent_light_position3 = tangent_matrix * lights[3].position;
    out.tangent_light_position4 = tangent_matrix * lights[4].position;
    out.tangent_light_position5 = tangent_matrix * lights[5].position;
    out.tangent_light_position6 = tangent_matrix * lights[6].position;
    out.tangent_light_position7 = tangent_matrix * lights[7].position;
    out.tangent_light_position8 = tangent_matrix * lights[8].position;
    out.tangent_light_position9 = tangent_matrix * lights[9].position;
    out.tangent_light_position10 = tangent_matrix * lights[10].position;
    out.tangent_light_position11 = tangent_matrix * lights[11].position;
    out.tangent_light_position12 = tangent_matrix * lights[12].position;
    out.tangent_light_position13 = tangent_matrix * lights[13].position;
    out.tangent_light_position14 = tangent_matrix * lights[14].position;
    out.tangent_light_position15 = tangent_matrix * lights[15].position;

    return out;
}

// Fragment shader

// This grabs the sampler from the Global uniform
@group(0) @binding(2)
var s_diffuse: sampler;
@group(0) @binding(3)
var s_normal: sampler;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var t_normal: texture_2d<f32>;


fn light_contribution(
    light: Light,
    tangent_light_position: vec3<f32>,
    tangent_position: vec3<f32>,
    tangent_normal: vec3<f32>,
    view_dir: vec3<f32>
) -> vec3<f32> {

    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    // Create the lighting vectors
    let light_dir = normalize(tangent_light_position - tangent_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    return ambient_color + diffuse_color + specular_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // Create the lighting vectors
    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);

    var result = vec3<f32>();
    // TODO: Use shader macro instead of repeating this code
    // TODO: Only compute lighting for active number of lights - the below assumes inactive lights
    // have zero contribution - which may not be the case if lights are removed and lights uniform 
    // buffer still contains old values instead of being zero'd out.

    result += light_contribution(lights[0], in.tangent_light_position0, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[1], in.tangent_light_position1, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[2], in.tangent_light_position2, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[3], in.tangent_light_position3, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[4], in.tangent_light_position4, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[5], in.tangent_light_position5, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[6], in.tangent_light_position6, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[7], in.tangent_light_position7, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[8], in.tangent_light_position8, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[9], in.tangent_light_position9, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[10], in.tangent_light_position10, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[11], in.tangent_light_position12, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[12], in.tangent_light_position12, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[13], in.tangent_light_position13, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[14], in.tangent_light_position14, in.tangent_position, tangent_normal, view_dir);
    result += light_contribution(lights[15], in.tangent_light_position15, in.tangent_position, tangent_normal, view_dir);

    result *= object_color.xyz;

#ifdef WEBXR
    let final_color = utils::gamma_correction(result);
#else
    let final_color = result;
#endif
    return vec4<f32>(final_color, object_color.a);
}