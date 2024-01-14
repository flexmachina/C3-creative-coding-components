use std::collections::HashMap;

use wgpu::{util::DeviceExt, BindGroupLayout, Device, Queue};

use crate::{
    camera::Camera,
    instance::{Instance, InstanceRaw},
    light::{Light, LightUniform},
    model::{self, DrawModel, Vertex},
    node::Node,
    pass::Pass,
    transform::Transform,
    Rect,
    shader_utils,
    texture,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub fn to_uniform(camera: &Camera, transform: &Transform) -> CameraUniform {
    CameraUniform {
        view_position: transform.position().to_homogeneous().into(),
        view_proj: camera.view_proj(&transform).into()
    }
}

pub struct PhongConfig {
    pub wireframe: bool,
}

pub struct PhongPass {
    // Common uniform buffers
    pub camera_buffer: wgpu::Buffer,
    pub light_buffer: wgpu::Buffer,
    // Instances
    instance_buffers: HashMap<usize, wgpu::Buffer>,
    // Phong pipeline
    pub phong_global_bind_group_layout: BindGroupLayout,
    pub phong_global_bind_group: wgpu::BindGroup,
    pub phong_local_bind_group_layout: BindGroupLayout,
    phong_local_bind_groups: HashMap<usize, wgpu::BindGroup>,
    pub phong_render_pipeline: wgpu::RenderPipeline,
    // Light pipeline
    pub light_global_bind_group_layout: BindGroupLayout,
    pub light_global_bind_group: wgpu::BindGroup,
    pub light_render_pipeline: wgpu::RenderPipeline,
}

impl PhongPass {
    pub fn new(
        phong_config: &PhongConfig,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        webxr: bool
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
                    // Light
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(light_size),
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

         // TODO: how do you make a zero buffer of size(LightUniform)?
        let temp_light = Light {position: [0.0, 0.0, 0.0].into(), color: [0.0, 0.0, 0.0].into()};
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("[Phong] Light"),
            contents: bytemuck::cast_slice(&[temp_light.to_uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

        // Setup the render pipeline
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
                // Light
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(light_size),
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
            front_face: if webxr {wgpu::FrontFace::Cw} else {wgpu::FrontFace::Ccw},
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        };
        let multisample = wgpu::MultisampleState {
            ..Default::default()
        };

        let mut shader_composer = shader_utils::init_composer();

        let phong_render_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                    label: Some("Phong Shader"),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                        shader_utils::load_shader!(&mut shader_composer, "phong.wgsl", webxr)
                ))};
                let shader_module = device.create_shader_module(shader_desc);

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
                    shader_utils::load_shader!(&mut shader_composer, "light.wgsl", webxr)
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
        // Create instance buffer
        let instance_buffers = HashMap::new();

        PhongPass {
            camera_buffer,
            light_buffer,
            instance_buffers,

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
}

impl Pass for PhongPass {
    fn draw(
        &mut self,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        device: &Device,
        queue: &Queue,
        nodes: &Vec<Node>,
        camera: (&Camera, &Transform),
        light: &Light,
        viewport: &Option<Rect>,
        clear_color: bool,
        clear_depth: bool
    ) -> wgpu::CommandBuffer {

        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[light.to_uniform()]),
        );

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[to_uniform(camera.0, camera.1)]),
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
            
            match viewport {
                Some(v) => { render_pass.set_viewport(v.x, v.y, v.w, v.h, 0.0, 1.0); }
                _ => {}
            };

            // Loop over the nodes/models in a scene and setup the specific models
            // local uniform bind group and instance buffers to send to shader
            // This is separate loop from the render because of Rust ownership
            // (can prob wrap in block instead to limit mutable use)

            for (model_index, node) in nodes.iter().enumerate() {
                // We create a bind group for each model's local uniform data
                // and store it in a hash map to look up later
                let phong_local_bind_group_layout = &self.phong_local_bind_group_layout;
                self.phong_local_bind_groups
                    .entry(model_index)
                    .or_insert_with(|| {
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("[Phong] Locals"),
                            layout: &phong_local_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(
                                        &node.model.materials[0].diffuse_texture.view,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(
                                        &node.model.materials[0].normal_texture.view,
                                    ),
                                },
                            ],
                        })
                    });

                // Setup instance buffer for the model
                // similar process as above using HashMap
                self.instance_buffers.entry(model_index).or_insert_with(|| {
                    // We condense the matrix properties into a flat array (aka "raw data")
                    // (which is how buffers work - so we can "stride" over chunks)
                    let instance_data = node
                        .instances
                        .iter()
                        .map(Instance::to_raw)
                        .collect::<Vec<_>>();
                    // Create the instance buffer with our data
                    let instance_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(&instance_data),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    instance_buffer
                });
            }

                        
            // Setup lighting pipeline
            render_pass.set_pipeline(&self.light_render_pipeline);    
            render_pass.set_bind_group(0, &self.light_global_bind_group, &[]);
            // TODO: fix this hack and define a model to use for lights
            render_pass.draw_model(&nodes[0].model);
            
            // Setup phong pipeline
            render_pass.set_pipeline(&self.phong_render_pipeline);
            render_pass.set_bind_group(0, &self.phong_global_bind_group, &[]);

            // Render/draw all nodes/models
            for (model_index, node) in nodes.iter().enumerate() {
                render_pass.set_vertex_buffer(1, self.instance_buffers[&model_index].slice(..));
                render_pass.set_bind_group(1, &self.phong_local_bind_groups[&model_index], &[]);
                // Draw all the model instances
                render_pass.draw_model_instanced(
                    &node.model,
                    0..*&node.instances.len() as u32
                );
            }
        }
        encoder.finish()
    }
}
