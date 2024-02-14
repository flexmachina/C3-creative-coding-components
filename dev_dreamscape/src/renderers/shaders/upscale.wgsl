struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@group(0)
@binding(0)
var src_texture: texture_2d<f32>;

@group(0)
@binding(1)
var src_sampler: sampler;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let texCoord = vs.uv;

    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    let srcTexelSize = vec2f(1.0) / vec2f(textureDimensions(src_texture));
    let filterRadius = max(srcTexelSize.x, srcTexelSize.y) * 3.;
    let x = filterRadius;
    let y = filterRadius;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    let a = textureSample(src_texture, src_sampler, vec2f(texCoord.x - x, texCoord.y + y)).rgb;
    let b = textureSample(src_texture, src_sampler, vec2f(texCoord.x,     texCoord.y + y)).rgb;
    let c = textureSample(src_texture, src_sampler, vec2f(texCoord.x + x, texCoord.y + y)).rgb;

    let d = textureSample(src_texture, src_sampler, vec2f(texCoord.x - x, texCoord.y)).rgb;
    let e = textureSample(src_texture, src_sampler, vec2f(texCoord.x,     texCoord.y)).rgb;
    let f = textureSample(src_texture, src_sampler, vec2f(texCoord.x + x, texCoord.y)).rgb;

    let g = textureSample(src_texture, src_sampler, vec2f(texCoord.x - x, texCoord.y - y)).rgb;
    let h = textureSample(src_texture, src_sampler, vec2f(texCoord.x,     texCoord.y - y)).rgb;
    let i = textureSample(src_texture, src_sampler, vec2f(texCoord.x + x, texCoord.y - y)).rgb;

    // Apply weighted distribution, by using a 3x3 tent filter:
    //  1   | 1 2 1 |
    // -- * | 2 4 2 |
    // 16   | 1 2 1 |
    var upsample = e*4.0;
    upsample += (b+d+f+h)*2.0;
    upsample += (a+c+g+i);
    upsample *= 1.0 / 16.0;
    return vec4f(upsample, 1.);
}
