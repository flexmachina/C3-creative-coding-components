/*
//use crate::assets::load_string;
use crate::device::Device;
use crate::texture::Texture;
use wgpu::util::DeviceExt;

//pub async fn new_shader_module(device: &Device, src_wgsl: &str) -> wgpu::ShaderModule {
pub fn new_shader_module(device: &Device, src_wgsl: &str) -> wgpu::ShaderModule {
    //let src = load_string(src_file_name).await.unwrap();

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        //source: wgpu::ShaderSource::Wgsl(src.into()),
        source: wgpu::ShaderSource::Wgsl(src_wgsl.into()), 
    })
}

pub fn new_uniform_bind_group(
    device: &Device,
    data: &[u8],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Buffer) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: data,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: None,
    });

    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: None,
    });

    (group_layout, group, buffer)
}

pub fn new_texture_bind_group(
    device: &Device,
    texture: &Texture,
    view_dimension: wgpu::TextureViewDimension,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture.view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(texture.sampler()),
            },
        ],
        label: None,
    });

    (layout, group)
}

pub struct RenderPipelineParams<'a> {
    pub shader_module: wgpu::ShaderModule,
    pub depth_write: bool,
    pub depth_enabled: bool,
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub vertex_buffer_layouts: &'a [wgpu::VertexBufferLayout<'a>],
}

pub fn new_render_pipeline(
//pub async fn new_render_pipeline(
    device: &Device,
    params: RenderPipelineParams<'_>,
) -> wgpu::RenderPipeline {
    let layout = device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: params.bind_group_layouts,
            push_constant_ranges: &[],
        });

    let pipeline = device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &params.shader_module,
                entry_point: "vs_main",
                buffers: params.vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &params.shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: device.surface_texture_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: if params.depth_enabled {
                Some(wgpu::DepthStencilState {
                    format: device.depth_texture_format(), // TODO Configurable
                    depth_write_enabled: params.depth_write,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

    pipeline
}
*/



// From https://github.com/MeetKai/superconductor/blob/3ed64c119e5e7419752e5602fe5d8868d5e503cf/renderer-core/src/lib.rs
// TODO: check below cfg usage is needed
//#[cfg(feature = "webgl")]

pub fn create_view_from_device_framebuffer(
    device: &wgpu::Device,
    framebuffer: web_sys::WebGlFramebuffer,
    base_layer: &web_sys::XrWebGlLayer,
    format: wgpu::TextureFormat,
    label: &'static str,
) -> wgpu::Texture  {
    unsafe {
        device.create_texture_from_hal::<wgpu_hal::gles::Api>(
            wgpu_hal::gles::Texture {
                inner: wgpu_hal::gles::TextureInner::ExternalFramebuffer { inner: framebuffer.clone() },
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
                view_formats: &[format],
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            },
        )
    }
}

