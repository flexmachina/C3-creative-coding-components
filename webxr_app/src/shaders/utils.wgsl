#define_import_path utils

#ifdef WEBXR
// When we render to an sRGB format frame buffer, it seems for WebXR
// we need to apply gamma correction.
// I'm not sure why this is, but the difference is that we're rendering directly
// to a WebGL managed frame buffer in WebXR mode, vs a wgpu managed one for desktop 
// and the regular WASM mode.
fn gamma_correction(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(1./2.2));
}
#endif
