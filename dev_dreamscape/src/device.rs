use std::ops::Deref;
use bevy_ecs::prelude::Resource;
use crate::texture::Texture;
use wgpu::Limits;

pub type SurfaceSize = winit::dpi::PhysicalSize<u32>;


#[derive(Resource)]
pub struct Device {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_tex: Texture,
}

impl Device {
    pub async fn new(window: &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits {
                            max_texture_dimension_2d: 4096,    
                            ..wgpu::Limits::downlevel_webgl2_defaults()
                        }
                    } else {
                        Limits {
                            max_texture_dimension_2d: 4096,     
                            ..wgpu::Limits::default()
                        }
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_size = window.inner_size();

        let surface_config = {
            let caps = surface.get_capabilities(&adapter);

            let format = caps
                .formats.iter()
                .copied()
                .find(|f| f.is_srgb())            
                .unwrap_or(caps.formats[0]);


            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: surface_size.width,
                height: surface_size.height,
                present_mode: caps.present_modes[0],
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
            }
        };
        surface.configure(&device, &surface_config);

        let depth_tex = Texture::create_depth_texture(&device, 
                            surface_size.width, surface_size.height, "depth_texture");


        Self {
            surface_config,
            surface,
            device,
            queue,
            depth_tex,
        }
    }

    pub fn resize(&mut self, new_size: SurfaceSize) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_tex = Texture::create_depth_texture(&self.device, 
                                    new_size.width, new_size.height, "depth_texture");
                //Texture::new_depth(&self.device, Self::DEPTH_TEX_FORMAT, new_size.into());
        }
    }

    pub fn surface_texture_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    pub fn surface_size(&self) -> SurfaceSize {
        SurfaceSize::new(self.surface_config.width, self.surface_config.height)
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn depth_tex(&self) -> &Texture {
        &self.depth_tex
    }
}

impl Deref for Device {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
