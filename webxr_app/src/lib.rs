mod assets;
mod camera;
mod instance;
mod light;
mod model;
mod shader_utils;
mod texture;
#[cfg(target_arch = "wasm32")]
mod utils;
mod wgpu_utils;
#[cfg(target_arch = "wasm32")]
mod xr;

use std::cell::RefCell;
use std::rc::Rc;

use cgmath::prelude::*;
#[allow(unused_imports)]
use log::{debug,error,info};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use camera::CameraState;
use model::{DrawLight, DrawModel, Vertex};
use instance::{Instance, InstanceRaw};

const NUM_INSTANCES_PER_ROW: u32 = 10;

struct RenderState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    color_format: wgpu::TextureFormat,
    depth_texture: texture::Texture,
    camera_state: CameraState,

    obj_model: model::Model,
    instances: Vec<Instance>,
    #[allow(dead_code)]
    instance_buffer: wgpu::Buffer,
    light: light::Light,
    light_render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    debug_material: model::Material,
}

pub struct WindowState
{
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    #[allow(dead_code)]
    cursor_pos: winit::dpi::PhysicalPosition<f64>,
    mouse_pressed: bool,
    window: Window,
}

pub struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32
}

pub struct App {
    state: Rc<RefCell<State>>,
    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    xr_app: Option<xr::XrApp>
}

impl App {
    async fn new(window: Window, webxr: bool) -> Self {
        // State::new uses async code, so we're going to wait for it to finish
        let state = State::new(window, webxr).await;
        let state = Rc::new(RefCell::new(state));
        #[cfg(target_arch = "wasm32")]
        {
            let xr_app = if webxr {
                let xr_app = xr::XrApp::new(state.clone());
                xr_app.init().await;
                Some(xr_app)
            } else {
                None
            };
            Self{state, xr_app}
        }        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self{state}
        }
    }
}
pub struct State {
    render_state: RenderState,
    window_state: WindowState
}

async fn create_redner_state(
    device: wgpu::Device,
    queue: wgpu::Queue,
    color_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    webxr: bool) -> RenderState
{
    let texture_bind_group_layout =
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // normal map
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });

    const SPACE_BETWEEN: f32 = 3.0;
    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    )
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>();

    let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX,
    });


    let obj_model: model::Model =
        assets::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();

    let light = light::Light::new(&device, [2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

    let debug_material = {
        let diffuse_bytes = include_bytes!("../res/cobble-diffuse.png");
        let normal_bytes = include_bytes!("../res/cobble-normal.png");

        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            diffuse_bytes,
            "res/alt-diffuse.png",
            false,
        )
        .unwrap();
        let normal_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            normal_bytes,
            "res/alt-normal.png",
            true,
        )
        .unwrap();

        model::Material::new(
            &device,
            "alt-material",
            diffuse_texture,
            normal_texture,
            &texture_bind_group_layout,
        )
    };

    let depth_texture = 
    texture::Texture::create_depth_texture(&device, width, height, "depth_texture");

    let camera_state = camera::CameraState::new(&device, width, height);

    let render_pipeline_layout =
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout,
            &camera_state.camera_bind_group_layout,
            &light.bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let mut shader_composer = shader_utils::init_composer();
    let render_pipeline = {
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Phong Shader"),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                shader_utils::load_shader!(&mut shader_composer, "phong.wgsl", webxr)
            ))
        };
        wgpu_utils::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            color_format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), InstanceRaw::desc()],
            shader,
            webxr
        )
    };

    let light_render_pipeline = {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline Layout"),
            bind_group_layouts: &[&camera_state.camera_bind_group_layout, &light.bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader"),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(
                shader_utils::load_shader!(&mut shader_composer, "light.wgsl", webxr)
            ))
        };
        wgpu_utils::create_render_pipeline(
            &device,
            &layout,
            color_format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc()],
            shader,
            webxr
        )
    };



    return RenderState { 
        device,
        queue, 
        render_pipeline,
        color_format,
        depth_texture,
        camera_state,

        obj_model,
        instances,
        instance_buffer,
        light,
        light_render_pipeline,
        debug_material,
    }
}

#[cfg(target_arch = "wasm32")]
fn setup_window_canvas(window: &Window) {

    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    use winit::dpi::PhysicalSize;
    window.set_inner_size(PhysicalSize::new(2048, 1024));

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("wasm-example")?;
            let canvas = web_sys::Element::from(window.canvas());
            canvas.set_id("canvas");
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

impl State {

    // Creating some of the wgpu types requires async code
    async fn new(window: Window, webxr: bool) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())            
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let cursor_pos = winit::dpi::PhysicalPosition {x:0.0, y:0.0};

        let render_state = create_redner_state(device, queue, surface_format, size.width, size.height, webxr).await;
        let window_state = WindowState{window, surface, config, size, cursor_pos, mouse_pressed: false};
        Self {
            render_state,
            window_state
        }
    }

    pub fn window(&self) -> &Window {
        let window_state = &self.window_state;
        &window_state.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.render_state.camera_state.projection.resize(new_size.width, new_size.height);
            let window_state = &mut self.window_state;
            window_state.size = new_size;
            window_state.config.width = new_size.width;
            window_state.config.height = new_size.height;
            window_state.surface.configure(&self.render_state.device, &window_state.config);
            self.render_state.depth_texture = texture::Texture::create_depth_texture(
                &self.render_state.device,
                new_size.width,
                new_size.height, 
                "depth_texture");
        }
    }
    
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.render_state.camera_state.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.render_state.camera_state.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.window_state.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        // Update camera
        self.render_state.camera_state.camera_controller.update_camera(&mut self.render_state.camera_state.camera, dt);
        self.render_state.camera_state.camera_uniform
            .update_view_proj(&self.render_state.camera_state.camera, &self.render_state.camera_state.projection);
        self.update_camera_buffer();
        
        self.update_scene(dt);
    }

    #[cfg(target_arch = "wasm32")]
    fn update_view_proj_webxr(&mut self, projection: &cgmath::Matrix4<f32>, pos: &cgmath::Vector3<f32>, rot: &cgmath::Quaternion<f32>) {
        self.render_state.camera_state.camera_uniform.update_view_proj_webxr(&projection, &pos, &rot);
        self.update_camera_buffer();
    }

    fn update_camera_buffer(&mut self) {
        self.render_state.queue.write_buffer(
            &self.render_state.camera_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.render_state.camera_state.camera_uniform]),
        );
    }

    fn update_scene(&mut self, dt: std::time::Duration) {
        // Update the light
        let old_position: cgmath::Vector3<_> = self.render_state.light.uniform.position.into();
        let deg_per_sec = 90.;
        let deg = cgmath::Deg(deg_per_sec * dt.as_secs_f32());
        self.render_state.light.uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), deg)
                * old_position)
                .into();
        self.render_state.queue.write_buffer(
            &self.render_state.light.buffer,
            0,
            bytemuck::cast_slice(&[self.render_state.light.uniform]),
        );
    }

    fn render_to_surface(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface = &self.window_state.surface;
        let output = surface.get_current_texture()?;
        self.render_to_texture(&output.texture, None, None, true);
        output.present();
        Ok(())
    }

    fn render_to_texture(&mut self, color_texture: &wgpu::Texture, depth_texture: Option<&wgpu::Texture>, viewport: Option<Rect>, clear: bool) {
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = match depth_texture {
            Some(d) => d.create_view(&wgpu::TextureViewDescriptor::default()),
            _ => self.render_state.depth_texture.texture.create_view(&wgpu::TextureViewDescriptor::default())
        };
        let mut encoder = self.render_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:
                            if clear {
                                wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                })
                            } else {
                                wgpu::LoadOp::Load
                            },
                        store: true,
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load:
                            if clear {
                                wgpu::LoadOp::Clear(1.0)
                            } else {
                                wgpu::LoadOp::Load
                            },
                        store: true
                    }),
                    stencil_ops: None,
                }),
            });

            match viewport {
                Some(v) => { render_pass.set_viewport(v.x, v.y, v.w, v.h, 0.0, 1.0); }
                _ => {}
            };

            render_pass.set_vertex_buffer(1, self.render_state.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_state.light_render_pipeline);
            render_pass.draw_light_model(
                &self.render_state.obj_model,
                &self.render_state.camera_state.camera_bind_group,
                &self.render_state.light.bind_group,
            );

            render_pass.set_pipeline(&self.render_state.render_pipeline);
            render_pass.draw_model_instanced(
                &self.render_state.obj_model,
                0..self.render_state.instances.len() as u32,
                &self.render_state.camera_state.camera_bind_group,
                &self.render_state.light.bind_group,
            );
        }

        // submit will accept anything that implements IntoIter
        self.render_state.queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run_windowed(false));
    }
    #[cfg(target_arch = "wasm32")]
    {
        utils::set_panic_hook();
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        const XR_MODE: bool = true;
        if XR_MODE {
            // Using future_to_promise here because spawn_local causes the xr.request_session to return a nil
            // on the Meta Quest browser (XrSession unwrap fails in xr.rs).
            // My best guess is that spawn_local interferes with the Meta Quest specific security check that ensures Immersive mode
            // can only be triggered via a user event. We see the same nil session behaviour when trying to 
            // enter immersive mode automatically when starting the WASM, instead of triggering it via a buttom 
            // click in index.html.
            // spawn_local(run_xr());
            let _ = wasm_bindgen_futures::future_to_promise(async {
                run_windowed(true).await;
                Ok(JsValue::UNDEFINED)
            });
        } else {
            wasm_bindgen_futures::spawn_local(run_windowed(false));
        }
    }
}

async fn run_windowed(webxr: bool) {    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Window setup...
    #[cfg(target_arch = "wasm32")]
    {
        setup_window_canvas(&window);
    }

    let app = App::new(window, webxr).await;
    let mut last_render_time = instant::Instant::now();

    // Create closure to handle events. Skip certain events in webxr mode, like surface drawing.
    let event_handler = move
        |event: Event<()> , _: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow | {
        *control_flow = ControlFlow::Poll;
        let mut state = app.state.borrow_mut();
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.window_state.mouse_pressed {
                state.render_state.camera_state.camera_controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() && !state.input(event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        if !webxr {
                            state.resize(*physical_size);
                        }
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        if !webxr {
                            state.resize(**new_inner_size);
                        }
                    }
                    WindowEvent::CursorMoved {..} => {
                        state.input(event);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                if !webxr {
                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    state.update(dt);
                    match state.render_to_surface() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let size = state.window_state.size;
                            state.resize(size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // We're ignoring timeouts
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                if !webxr {
                    state.window().request_redraw();
                }
            }
            _ => {}
        }
    };
    #[cfg(target_arch = "wasm32")]
    {
        event_loop.spawn(event_handler);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop.run(event_handler);
    }
}

