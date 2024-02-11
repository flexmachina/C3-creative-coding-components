use wgpu::Operations;

use crate::texture;

use super::{shader_utils, utils};

// We could use `Rgba32Float`, but that requires some extra
// features to be enabled.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

        
pub struct DownscalePipeline {
    downscale_pipeline: wgpu::RenderPipeline,
    upscale_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>,
    textures: Vec<texture::Texture>,
    levels: u32,
}

impl DownscalePipeline {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        levels: u32,
    ) -> Self {

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Downscale::layout"),
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

        let textures = Self::create_textures(&device, width, height, levels);
        let bind_groups = Self::create_bind_groups(&device, &textures, &bind_group_layout);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let mut shader_composer = shader_utils::init_composer();

        let downscale_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                    label: Some("Downscale::shader"),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                        shader_utils::load_shader!(&mut shader_composer, "downscale.wgsl", None)
                ))};
            utils::create_render_pipeline(
                device,
                &pipeline_layout,
                "Downscale::pipeline",
                FORMAT,
                None,
                &[],
                shader_desc,
            )
        };

        let upscale_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                    label: Some("Upscale::shader"),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                        shader_utils::load_shader!(&mut shader_composer, "upscale.wgsl", None)
                ))};
            utils::create_render_pipeline(
                device,
                &pipeline_layout,
                "Upscale::pipeline",
                FORMAT,
                None,
                &[],
                shader_desc,
            )
        };

        Self {
            downscale_pipeline,
            upscale_pipeline,
            bind_group_layout,
            bind_groups,
            textures,
            levels,
        }
    }

    fn create_textures(device: &wgpu::Device, width: u32, height: u32, levels: u32) -> Vec<texture::Texture> {
        let mut textures: Vec<texture::Texture> = vec![];
        for level in 0..levels {
            let w = width / (2u32.pow(level));
            let h = height / (2u32.pow(level));
            let texture = texture::Texture::create_2d_texture(
                device,
                w,
                h,
                FORMAT,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                wgpu::FilterMode::Linear,
                Some("Downscale::textures")
            );
            textures.push(texture)
        }
        textures
    }

    fn create_bind_groups(
            device: &wgpu::Device, 
            textures: &Vec<texture::Texture>,
            bind_group_layout: &wgpu::BindGroupLayout
        ) -> Vec<wgpu::BindGroup> {
        let mut bind_groups: Vec<wgpu::BindGroup> = vec![];
        for texture in textures.iter() {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Downscale::bind_group"),
                layout: &bind_group_layout,
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
            bind_groups.push(bind_group);
        }
        bind_groups
    }

    // Resize the texture stack if needed
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.textures.is_empty() || self.textures[0].texture.width() != width || self.textures[0].texture.height() != height {
            log::warn!("Downscale::resize textures");
            self.textures = Self::create_textures(device, width, height, self.levels);
            self.bind_groups = Self::create_bind_groups(device, &self.textures, &self.bind_group_layout);
        }
    }

    pub fn top_texture(&self) -> &wgpu::Texture {
        &self.textures[0].texture
    }

    pub fn bottom_texture(&self) -> &wgpu::Texture {
        &self.textures[1].texture
    }

    pub fn top_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_groups[0]
    }

    pub fn bottom_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_groups[1]
    }

    // This renders the internal HDR texture to the supplied TextureView
    // The viewport is supplied in WebXR mode
    pub fn process(&self, device: &wgpu::Device) -> wgpu::CommandBuffer {

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Downscale::command_encoder"),
        });
        
        {
            // Downscale
            for i in 0..self.textures.len()-1 {
                let dest = &self.textures[i+1].view;

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Downscale::render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &dest,
                        resolve_target: None,
                        ops: Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                pass.set_pipeline(&self.downscale_pipeline);
                pass.set_bind_group(0, &self.bind_groups[i], &[]);
                pass.draw(0..3, 0..1);
            }

            // Upscale and blur. Leave topmost texture unchanged as this is our original render
            for i in (2..self.textures.len()).rev() {
                let dest = &self.textures[i-1].view;

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Downscale::render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &dest,
                        resolve_target: None,
                        ops: Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                pass.set_pipeline(&self.upscale_pipeline);
                pass.set_bind_group(0, &self.bind_groups[i], &[]);
                pass.draw(0..3, 0..1);
            }        
        }

        encoder.finish()
    }
}
