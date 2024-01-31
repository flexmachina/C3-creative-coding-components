use std::collections::HashMap;

use naga_oil::compose::ShaderDefValue;
use wgpu::{BindGroupLayout, Queue};

use crate::{
    components::{Camera, Light, Transform},
    device::Device,
    model,
    model::{DrawModel, Model, Vertex},
    texture,
};

use super::{
    shader_utils,
    instance,
    instance::InstanceRaw,
};


const MAX_LIGHTS: u64 = 4;

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub fn camera_uniform(camera: &Camera, transform: &Transform) -> CameraUniform {
    CameraUniform {
        view_position: transform.position().to_homogeneous().into(),
        view_proj: camera.view_proj(&transform).into()
    }
} 

// Uniform for light data (position + color)
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

pub fn light_uniform(light: &Light, transform: &Transform) -> LightUniform {
    LightUniform {
        position: transform.position().into(),
        _padding: 0,
        color: light.color.into(),
        _padding2: 0,
    }
}

pub struct PhongConfig {
    pub wireframe: bool,
}

pub struct PhongPass {
    // Common uniform buffers
    pub camera_buffer: wgpu::Buffer,
    pub light_buffer: wgpu::Buffer,
    // Instance buffer pool - keyed by node index
    instance_buffers: HashMap<usize, wgpu::Buffer>,
    // Phong pipeline
    pub phong_global_bind_group_layout: BindGroupLayout,
    pub phong_global_bind_group: wgpu::BindGroup,
    pub phong_local_bind_group_layout: BindGroupLayout,
    // Bind groups - keyed by model
    // TODO: make ModelSpec / Material id the key
    phong_local_bind_groups: HashMap<String, wgpu::BindGroup>,
    pub phong_render_pipeline: wgpu::RenderPipeline,
    // Light pipeline
    pub light_global_bind_group_layout: BindGroupLayout,
    pub light_global_bind_group: wgpu::BindGroup,
    pub light_render_pipeline: wgpu::RenderPipeline,
}

impl PhongPass {
    pub fn new(
        phong_config: &PhongConfig,
        device: &Device,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        // Setup global uniforms
        // Global bind group layout
        let light_size = std::mem::size_of::<LightUniform>() as wgpu::BufferAddress;
        let camera_size = std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress;
        let phong_global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Globals"),
                entries: &[
                    // Camera
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(camera_size),
                        },
                        count: None,
                    },
                    // Lights
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(light_size * MAX_LIGHTS),
                        },
                        count: None,
                    },
                    // Sampler for diffuse texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Sampler for normal texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Global uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("[Phong] Camera"),
            size: camera_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("[Phong] Light"),
            size: light_size * MAX_LIGHTS,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // We also need a sampler for our textures
        // TODO: different sampler for normals??
        let sampler_diffuse = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("[Phong] diffuse sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let sampler_normal = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("[Phong] normal sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // Combine the global uniform, the lights, and the texture sampler into one bind group
        let phong_global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[Phong] Globals"),
            layout: &phong_global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler_diffuse),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler_normal),
                },
            ],
        });

        // Setup local uniforms
        // Local bind group layout
        let phong_local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Locals"),
                entries: &[
                    // Diffuse texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Normal texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Setup the render pipelines
        let phong_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("[Phong] Pipeline"),
            bind_group_layouts: &[&phong_global_bind_group_layout, &phong_local_bind_group_layout],
            push_constant_ranges: &[],
        });

        let light_global_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("[Light] Globals"),
            entries: &[
                // Camera
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(camera_size),
                    },
                    count: None,
                },
                // Lights
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(light_size * MAX_LIGHTS),
                    },
                    count: None,
                }
            ]
        });

        let light_global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[Light] Globals"),
            layout: &light_global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                }
            ],
        });

        let light_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("[Light] Pipeline"),
            bind_group_layouts: &[&light_global_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [model::ModelVertex::desc(), InstanceRaw::desc()];
        let depth_stencil = Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        });

        // Enable/disable wireframe mode
        let topology = if phong_config.wireframe {
            wgpu::PrimitiveTopology::LineList
        } else {
            wgpu::PrimitiveTopology::TriangleList
        };

        let primitive = wgpu::PrimitiveState {
            topology,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        };
        let multisample = wgpu::MultisampleState {
            ..Default::default()
        };

        let mut shader_composer = shader_utils::init_composer();
        let shader_defs = HashMap::from([
            ("MAX_LIGHTS".to_string(), ShaderDefValue::Int(MAX_LIGHTS as i32))
        ]);
        let phong_render_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                    label: Some("Phong Shader"),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                        shader_utils::load_shader!(&mut shader_composer, "phong.wgsl", Some(shader_defs.clone()))
                ))};
                let shader_module = device.create_shader_module(shader_desc);

                // TODO: Use utils::create_render_pipeline
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("[Phong] Pipeline"),
                layout: Some(&phong_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                primitive,
                depth_stencil: depth_stencil.clone(),
                multisample,
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
        };

        let light_render_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                    shader_utils::load_shader!(&mut shader_composer, "light.wgsl", Some(shader_defs.clone()))
                ))
            };
            let shader_module = device.create_shader_module(shader_desc);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("[Light] Pipeline"),
                layout: Some(&light_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[model::ModelVertex::desc()],
                },
                primitive,
                depth_stencil,
                multisample,
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
        };

        PhongPass {
            camera_buffer,
            light_buffer,
            instance_buffers: Default::default(),

            phong_global_bind_group_layout,
            phong_global_bind_group,
            phong_local_bind_group_layout,
            phong_local_bind_groups: Default::default(),
            phong_render_pipeline,
            
            light_global_bind_group,
            light_global_bind_group_layout,
            light_render_pipeline,
        }
    }

    pub fn draw(
        &mut self,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        device: &Device,
        queue: &Queue,
        nodes: &Vec<(&Model, &String, Vec<&Transform>)>,
        camera: (&Camera, &Transform),
        lights: &Vec<(&Light, &Transform)>,
        light_model: &Model,
        clear_color: bool,
        clear_depth: bool
    ) -> wgpu::CommandBuffer {

        assert!(lights.len() <= MAX_LIGHTS as usize);

        let lights_data = lights
            .iter()
            .map(|l| light_uniform(l.0, l.1))
            .collect::<Vec<_>>();

        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&lights_data),
        );

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform(camera.0, camera.1)]),
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Setup the render pass
        // see: clear color, depth stencil
        {
           let mut render_pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load:
                            if clear_depth {wgpu::LoadOp::Clear(1.0)} 
                            else { wgpu::LoadOp::Load},
                        store: true
                    }),
                    stencil_ops: None,
                }),
            });
            
            let instance_size = std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress;

            // Loop over the nodes  and setup model specific bind groups and 
            // instance buffers per node to send to shader
            // This is separate loop from the render because of Rust ownership
            // (can prob wrap in block instead to limit mutable use)
            for (model_index, (model, modelname, transforms)) in nodes.iter().enumerate() {
                // We create a bind group for each model's local uniform data
                // and store it in a hash map to look up later
                
                //
                // Bindgroup  management

                // Bindgroups are indexed by modelname as we currently assume models have a fixed material
                // (ideally we should index by a Material id)
                let phong_local_bind_group_layout = &self.phong_local_bind_group_layout;
                self.phong_local_bind_groups
                    .entry(modelname.to_string())
                    .or_insert_with(|| {
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("[Phong] Locals"),
                            layout: &phong_local_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(
                                        &model.materials[0].diffuse_texture.view,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(
                                        &model.materials[0].normal_texture.view,
                                    ),
                                },
                            ],
                        })
                    });
                
                //
                // Instance buffer management
                //

                // Instance buffers are indexed by node, as we need a separate instance buffer per node.
                // We could just use a Vec, but then we'd have to manage resizing it.
                // The Hasmap takes care of this for us
                // TODO: Should be shrink the HashMap and contained buffers in a cleanup routine? e.g.
                // when changing scenes
                let required_instance_buffer_size = instance_size * transforms.len() as u64;

                let create_buffer = || device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instance Buffer"),
                    size:  required_instance_buffer_size,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false
                });

                // Get instance buffer keyed by model or create one if it doesn't exist 
                let instance_buffer = self.instance_buffers
                    .entry(model_index)
                    .or_insert_with(|| {
                        create_buffer()
                    });
                // Reallocate the instance buffer if it's too small
                if instance_buffer.size() < required_instance_buffer_size as u64 {
                    // Reallocate global instance buffer
                    *instance_buffer = create_buffer();
                }
                // Write to the instance buffer
                let instance_data = transforms.iter().map(instance::instance_raw).collect::<Vec<_>>();
                queue.write_buffer(
                    &instance_buffer,
                    0,
                    bytemuck::cast_slice(&instance_data),
                );
            }
                        
            // Setup lighting pipeline
            render_pass.set_pipeline(&self.light_render_pipeline);    
            render_pass.set_bind_group(0, &self.light_global_bind_group, &[]);

            // Draw lights. Assume a single model which conveniently allows us to use
            // instancing where the the instance_index can be used to index into
            // to Lights array uniform buffer in the shader.
            render_pass.draw_model_instanced(light_model, 0..lights.len() as u32);
            
            // Setup phong pipeline
            render_pass.set_pipeline(&self.phong_render_pipeline);
            render_pass.set_bind_group(0, &self.phong_global_bind_group, &[]);

            // Draw all node models
            for (model_index, (model, modelname, transforms)) in nodes.iter().enumerate() {
                let required_instance_buffer_size = instance_size * transforms.len() as u64;
                let instance_buffer = self.instance_buffers.get(&model_index).unwrap();
                // It looks like we don't need to limit the bounds of the instance buffer slice,
                // (probably because instance range passed to draw_model_instanced defines how much of the 
                // buffer is read, but doing it anyway for sanity purposes.
                render_pass.set_vertex_buffer(1, instance_buffer.slice(0..required_instance_buffer_size));
                render_pass.set_bind_group(1, &self.phong_local_bind_groups[*modelname], &[]);
                // Draw all the model instances
                render_pass.draw_model_instanced(
                    &model,
                    0..transforms.len() as u32
                );
            }
        }
        encoder.finish()
    }
}
