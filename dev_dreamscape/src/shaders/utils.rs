use crate::assets::load_string;
use crate::device::Device;
use crate::texture::Texture;
use wgpu::util::DeviceExt;

pub async fn new_shader_module(device: &Device, src_file_name: &str) -> wgpu::ShaderModule {
    let src = load_string(src_file_name).await.unwrap();

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(src.into()),
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

pub async fn new_render_pipeline(
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
