use std::collections::HashMap;

use wgpu::util::DeviceExt;
use crate::{
    components::{Camera,Transform},
    device::Device,
    math::Mat4,
    renderers::shader_utils,
    texture::Texture,
};





#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    view_proj_inv: [[f32; 4]; 4],
}

pub struct SkyboxPass {
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<String, wgpu::BindGroup>,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl SkyboxPass {
    pub fn new(
        device: &Device,
        color_format: wgpu::TextureFormat,
    ) -> Self {

        let uniform = Uniform{view_proj_inv: Mat4::identity().into()};

        let (uniform_bind_group_layout, uniform_bind_group, uniform_buffer) =
            new_uniform_bind_group(device, bytemuck::cast_slice(&[uniform]));

        let texture_bind_group_layout = 
            new_texture_bind_group_layout(device, wgpu::TextureViewDimension::Cube);

        let pipeline_layout = device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[] 
            });
        
        let mut shader_composer = shader_utils::init_composer();
        let render_pipeline = {

            let primitive = wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            };

            let multisample = wgpu::MultisampleState {
                ..Default::default()
            };
            
            let shader_desc = wgpu::ShaderModuleDescriptor {
                    label: Some("Skybox Shader"),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                        shader_utils::load_shader!(&mut shader_composer, "skybox.wgsl", None)
                ))};
                let shader_module = device.create_shader_module(shader_desc);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("[Skybox] Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    // No geometry. Renders to screen space triangle
                    buffers: &[],
                },
                primitive,
                depth_stencil: None,
                multisample,
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
        };

        Self {
            render_pipeline,
            texture_bind_group_layout,
            texture_bind_groups: Default::default(),
            uniform_buffer,
            uniform_bind_group,
        }
    }
}

impl SkyboxPass {
    pub fn draw(
        &mut self,
        color_view: &wgpu::TextureView,
        device: &Device,
        camera: (&Camera, &Transform),
        texture: (&str, &Texture),
        clear_color: bool
    ) -> wgpu::CommandBuffer {

        // Compute matrix that goes from screen space (fullscreen quad coordinate) to direction in world space,
        // for sampling the cubemap texture.
        // Use inverse of view matrix (i.e. the camera matrix), without translation.
        let mut view_inv = camera.1.matrix();
        view_inv.append_translation_mut(&-camera.1.position()); // set translation to 0
        let view_proj_inv = view_inv * camera.0.inv_projection_matrix();
        let uniform = Uniform { 
            view_proj_inv: view_proj_inv.into()
        };

        device.queue().write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );

        // Bindgroup  management
        let texture_bind_group_layout = &self.texture_bind_group_layout;
        let texture_bind_group = self.texture_bind_groups
            .entry(texture.0.to_string())
            .or_insert_with(|| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("[Skybox] Texture"),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.1.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture.1.sampler),
                        },
                    ],
                })
            });
            
    
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Skybox Encoder"),
        });

        // Setup the render pass
        // see: clear color, depth stencil
        {
           let mut render_pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Skybox Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:
                            if clear_color {wgpu::LoadOp::Clear(wgpu::Color::BLACK) }
                            else { wgpu::LoadOp::Load },
                        store: true,
                    }
                })],
                depth_stencil_attachment: None
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &texture_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        encoder.finish()
    }
}

pub fn new_texture_bind_group_layout(
    device: &Device,
    view_dimension: wgpu::TextureViewDimension,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
