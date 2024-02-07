use std::collections::HashMap;
use naga_oil::compose::ShaderDefValue;
use wgpu::Operations;

use crate::{math::Rect, texture};

use super::{shader_utils, utils};

/// Owns the render texture and controls tonemapping
pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    width: u32,
    height: u32,
    layout: wgpu::BindGroupLayout,

    depth_texture: texture::Texture 
}

impl HdrPipeline {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        output_color_format: wgpu::TextureFormat,
        webxr: bool
    ) -> Self {

        let texture = Self::create_color_texture(device, width, height);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hdr::layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        // The Rgba16Float format cannot be filtered
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = {
            let shader_defs = if webxr {
                Some(HashMap::from([("WEBXR".to_string(),  ShaderDefValue::Bool(true))]))
            } else {
                None
            };
            let mut shader_composer = shader_utils::init_composer();
            let shader_desc = wgpu::ShaderModuleDescriptor {
                    label: Some("Hdr::shader"),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                        shader_utils::load_shader!(&mut shader_composer, "hdr.wgsl", shader_defs)
                ))};
            utils::create_render_pipeline(
                device,
                &pipeline_layout,
                "Hdr::pipeline",
                output_color_format,
                None,
                &[],
                shader_desc,
            )
        };

        let depth_texture = texture::Texture::create_depth_texture(&device, width, height, "depth_texture");

        Self {
            pipeline,
            bind_group,
            layout,
            texture,
            width,
            height,
            depth_texture
        }
    }

    fn create_color_texture(device: &wgpu::Device, width: u32, height: u32) -> texture::Texture {
        // We could use `Rgba32Float`, but that requires some extra
        // features to be enabled.
        let format = wgpu::TextureFormat::Rgba16Float;
        texture::Texture::create_2d_texture(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("Hdr::texture")
        )
    }

    // Resize the colour and depth textures if needed
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.texture.texture.width() != width || self.texture.texture.height() != height {
            log::warn!("Resizing hdr color texture: {}x{}", width, height);
            self.texture = Self::create_color_texture(device, width, height);
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Hdr::bind_group"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                    },
                ],
            });
        }
        if self.depth_texture.texture.width() != width || self.depth_texture.texture.height() != height {  
            log::warn!("Resizing hdr depth texture: {}x{}", width, height);
            self.depth_texture = texture::Texture::create_depth_texture(&device, width, height, "depth_texture");
        }
        self.width = width;
        self.height = height;
    }

    /// Exposes the HDR texture
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture.texture
    }

    /// The format of the HDR texture
    pub fn format(&self) -> wgpu::TextureFormat {
        self.texture.texture.format()
    }

    /// Exposes the HDR texture
    pub fn depth_texture(&self) -> &wgpu::Texture {
        &self.depth_texture.texture
    }

    // This renders the internal HDR texture to the supplied TextureView
    // The viewport is supplied in WebXR mode
    pub fn process(&self, device: &wgpu::Device, output: &wgpu::TextureView, viewport: Option<Rect>) -> wgpu::CommandBuffer {

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Hdr::command_encoder"),
        });
        
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Hdr::render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            match viewport {
                Some(v) => { pass.set_viewport(v.x, v.y, v.w, v.h, 0.0, 1.0); }
                _ => {}
            };

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        encoder.finish()
    }
}

