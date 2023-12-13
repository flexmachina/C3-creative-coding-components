mod assets;
mod camera;
mod instance;
mod light;
mod model;
mod node;
mod pass;
mod phong;
mod shader_utils;
mod texture;
#[cfg(target_arch = "wasm32")]
mod utils;
#[cfg(target_arch = "wasm32")]
mod xr;

use std::cell::RefCell;
use std::rc::Rc;

use cgmath::prelude::*;
#[allow(unused_imports)]
use log::{debug,error,info};
use pass::Pass;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use camera::CameraState;
use instance::Instance;
use node::Node;

use crate::phong::PhongConfig;

const NUM_INSTANCES_PER_ROW: u32 = 10;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32
}

struct RenderState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    #[allow(dead_code)]
    color_format: wgpu::TextureFormat,
    depth_texture: texture::Texture,
    camera_state: CameraState,

    phong_pass: phong::PhongPass,
    nodes: Vec<Node>,

    light: light::Light,
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

    let obj_model: model::Model =
        assets::load_model("cube.obj", &device, &queue)
            .await
            .unwrap();

    let light = light::Light::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

    let obj_node = Node {
        model: obj_model,
        instances: instances
    };
    let nodes = vec![obj_node];

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
            "alt-material",
            diffuse_texture,
            normal_texture
        )
    };

    let depth_texture = 
    texture::Texture::create_depth_texture(&device, width, height, "depth_texture");

    let camera_state = camera::CameraState::new(width, height);

    let phong_config = PhongConfig {
        max_lights: 1,
        ambient: Default::default(),
        wireframe: false,
    };
    let phong_pass = phong::PhongPass::new(
        &phong_config,
        &device,
        color_format,
        webxr
    );

    return RenderState { 
        device,
        queue, 
        color_format,
        depth_texture,
        camera_state,

        phong_pass,
        nodes,
 
        light,
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
            self.render_state.camera_state.camera.projection.resize(new_size.width, new_size.height);
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
            .update_view_proj(&self.render_state.camera_state.camera);
        self.update_scene(dt);
    }

    #[cfg(target_arch = "wasm32")]
    fn update_view_proj_webxr(&mut self, projection: &cgmath::Matrix4<f32>, pos: &cgmath::Vector3<f32>, rot: &cgmath::Quaternion<f32>) {
        self.render_state.camera_state.camera_uniform.update_view_proj_webxr(&projection, &pos, &rot);
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
        let command_buffer  = self.render_state.phong_pass.draw(
            &color_view,
            &depth_view,
            &self.render_state.device,
            &self.render_state.queue,
            &self.render_state.nodes,
            &self.render_state.camera_state.camera_uniform,
            &self.render_state.light.uniform,
            viewport,
            clear
        );
        
        // submit will accept anything that implements IntoIter
        self.render_state.queue.submit(std::iter::once(command_buffer));
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

