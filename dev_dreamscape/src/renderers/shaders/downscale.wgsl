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
    let srcTexelSize = vec2f(1.0) / vec2f(textureDimensions(src_texture));
    let x = srcTexelSize.x;
    let y = srcTexelSize.y;

    // Take 13 samples around current texel:
    // a - b - c
    // - j - k -
    // d - e - f
    // - l - m -
    // g - h - i
    // === ('e' is the current texel) ===
    let a = textureSample(src_texture, src_sampler, vec2f(texCoord.x - 2.*x, texCoord.y + 2.*y)).rgb;
    let b = textureSample(src_texture, src_sampler, vec2f(texCoord.x,        texCoord.y + 2.*y)).rgb;
    let c = textureSample(src_texture, src_sampler, vec2f(texCoord.x + 2.*x, texCoord.y + 2.*y)).rgb;

    let d = textureSample(src_texture, src_sampler, vec2f(texCoord.x - 2.*x, texCoord.y)).rgb;
    let e = textureSample(src_texture, src_sampler, vec2f(texCoord.x,        texCoord.y)).rgb;
    let f = textureSample(src_texture, src_sampler, vec2f(texCoord.x + 2.*x, texCoord.y)).rgb;

    let g = textureSample(src_texture, src_sampler, vec2f(texCoord.x - 2.*x, texCoord.y - 2.*y)).rgb;
    let h = textureSample(src_texture, src_sampler, vec2f(texCoord.x,        texCoord.y - 2.*y)).rgb;
    let i = textureSample(src_texture, src_sampler, vec2f(texCoord.x + 2.*x, texCoord.y - 2.*y)).rgb;

    let j = textureSample(src_texture, src_sampler, vec2f(texCoord.x - x,    texCoord.y + y)).rgb;
    let k = textureSample(src_texture, src_sampler, vec2f(texCoord.x + x,    texCoord.y + y)).rgb;
    let l = textureSample(src_texture, src_sampler, vec2f(texCoord.x - x,    texCoord.y - y)).rgb;
    let m = textureSample(src_texture, src_sampler, vec2f(texCoord.x + x,    texCoord.y - y)).rgb;

    // Apply weighted distribution:
    // 0.5 + 0.125 + 0.125 + 0.125 + 0.125 = 1
    // a,b,d,e * 0.125
    // b,c,e,f * 0.125
    // d,e,g,h * 0.125
    // e,f,h,i * 0.125
    // j,k,l,m * 0.5
    // This shows 5 square areas that are being sampled. But some of them overlap,
    // so to have an energy preserving downsample we need to make some adjustments.
    // The weights are the distributed, so that the sum of j,k,l,m (e.g.)
    // contribute 0.5 to the final color output. The code below is written
    // to effectively yield this sum. We get:
    // 0.125*5 + 0.03125*4 + 0.0625*4 = 1
    var downsample = e*0.125;
    downsample += (a+c+g+i)*0.03125;
    downsample += (b+d+f+h)*0.0625;
    downsample += (j+k+l+m)*0.125;
    return vec4f(downsample, 1.);
}
