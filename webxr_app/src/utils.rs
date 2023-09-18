// From https://github.com/MeetKai/superconductor/blob/3ed64c119e5e7419752e5602fe5d8868d5e503cf/renderer-core/src/lib.rs
// TODO: check below cfg usage is needed
#[cfg(feature = "webgl")]
pub fn create_view_from_device_framebuffer(
    device: &wgpu::Device,
    framebuffer: web_sys::WebGlFramebuffer,
    base_layer: &web_sys::XrWebGlLayer,
    format: wgpu::TextureFormat,
    label: &'static str,
) -> webgpu::Texture {
    device.create_texture_from_hal::<wgpu_hal::gles::Api>(
        wgpu_hal::gles::Texture {
            inner: wgpu_hal::gles::TextureInner::ExternalFramebuffer { inner: framebuffer },
            mip_level_count: 1,
            array_layer_count: 1,
            format,
            format_desc: wgpu_hal::gles::TextureFormatDesc {
                internal: glow::RGBA,
                external: glow::RGBA,
                data_type: glow::UNSIGNED_BYTE,
            },
            copy_size: wgpu_hal::CopyExtent {
                width: base_layer.framebuffer_width(),
                height: base_layer.framebuffer_height(),
                depth: 1,
            },
            is_cubemap: false,
            drop_guard: None,
        },
        &wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: base_layer.framebuffer_width(),
                height: base_layer.framebuffer_height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        },
    )
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
