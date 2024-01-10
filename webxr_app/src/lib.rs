mod assets;
mod camera;
mod camera_controller;
mod instance;
mod light;
mod maths;
mod model;
mod node;
mod pass;
mod phong;
mod shader_utils;
mod skybox;
mod texture;
#[cfg(target_arch = "wasm32")]
mod utils;
mod wgpu_ext;
#[cfg(target_arch = "wasm32")]
mod xr;

use std::cell::RefCell;
use std::rc::Rc;

#[allow(unused_imports)]
use log::{debug,error,info};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use camera_controller::CameraController;
use instance::Instance;
use maths::{Vec3, UnitQuat, UnitVec3};
use node::Node;
use pass::Pass;
use phong::PhongConfig;

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
    #[allow(dead_code)]
    skybox_texture: texture::Texture,
    phong_pass: phong::PhongPass,
    skybox_pass: skybox::SkyboxPass,
}

// Entities in world
struct Scene {
    light: light::Light,
    camera: camera::Camera,
    nodes: Vec<Node>,
}

pub struct WindowState {
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
    window_state: WindowState,
    camera_controller: CameraController,
    scene: Scene,
}

async fn create_scene(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32) -> Scene {

    let light = light::Light{
        position: [2.0, 2.0, 2.0].into(),
        color: [1.0, 1.0, 1.0].into()
    };

    let projection = camera::Projection::new(
        width, height, 
        45.0, 0.1, 100.0);
    let camera = camera::Camera::new(
        [0.0, 0.0, 0.0], 
        0.0, 
        0.0,
        projection);

    const SPACE_BETWEEN: f32 = 3.0;
    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = Vec3::new(x, 0.0, z);

                let rotation = if position == Vec3::zeros() {
                    UnitQuat::from_axis_angle(
                        &Vec3::z_axis(),
                        0.0,
                    )
                } else {
                    let angle: f32 = 45.0;
                    UnitQuat::from_axis_angle(&UnitVec3::new_normalize(position), angle.to_radians())
                };

                Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>();

    // TODO: Use resource id instead?
    let obj_model: model::Model =
        assets::load_model("cube.obj", &device, &queue)
            .await
            .unwrap();

    let obj_node = Node {
        model: obj_model,
        instances: instances
    };
    let nodes = vec![obj_node];

    Scene {
        light,
        camera,
        nodes
    }
}

async fn create_redner_state(
    device: wgpu::Device,
    queue: wgpu::Queue,
    color_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    webxr: bool) -> RenderState {

    let depth_texture = 
    texture::Texture::create_depth_texture(&device, width, height, "depth_texture");

    let phong_config = PhongConfig {
        wireframe: false,
    };
    let phong_pass = phong::PhongPass::new(
        &phong_config,
        &device,
        color_format,
        webxr
    );

    let skybox_texture = texture::Texture::load_cubemap_from_pngs("skyboxes/planet_atmosphere", &device, &queue).await;
    //let skybox_texture = texture::Texture::load_cubemap_from_ktx2("skyboxes/lake", &device, &queue).await;
    let skybox_config = skybox::SkyboxConfig {
        texture: &skybox_texture
    };
    
    let skybox_pass = skybox::SkyboxPass::new(
        &device,
        &skybox_config,
        color_format,
        webxr
    );

    return RenderState { 
        device,
        queue, 
        color_format,
        depth_texture,
        skybox_texture,
        phong_pass,
        skybox_pass,
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

        let scene = create_scene(&device, &queue, size.width, size.height).await;

        let render_state = create_redner_state(device, queue, surface_format, size.width, size.height, webxr).await;
        let window_state = WindowState{window, surface, config, size, cursor_pos, mouse_pressed: false};
        let camera_controller = CameraController::new(4.0, 0.4);
        Self {
            render_state,
            window_state,
            camera_controller,
            scene
        }
    }

    pub fn window(&self) -> &Window {
        let window_state = &self.window_state;
        &window_state.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.scene.camera.projection.resize(new_size.width, new_size.height);
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
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
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
        self.camera_controller.update_camera(&mut self.scene.camera, dt);
        self.update_scene(dt);
    }

    fn update_scene(&mut self, dt: std::time::Duration) {
        // Update the light
        let old_position= self.scene.light.position;
        let deg_per_sec = 90.;
        let deg = deg_per_sec * dt.as_secs_f32();
        self.scene.light.position =
            UnitQuat::from_axis_angle(&Vec3::y_axis(), deg.to_radians())
                * old_position;
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
        let command_buffer1 = self.render_state.skybox_pass.draw(
            &color_view,
            &depth_view,
            &self.render_state.device,
            &self.render_state.queue,
            &self.scene.nodes,
            &self.scene.camera,
            &self.scene.light,
            &viewport,
            clear,
            clear,
        );
        
        let command_buffer2 = self.render_state.phong_pass.draw(
            &color_view,
            &depth_view,
            &self.render_state.device,
            &self.render_state.queue,
            &self.scene.nodes,
            &self.scene.camera,
            &self.scene.light,
            &viewport,
            false,
            clear);

            // submit will accept anything that implements IntoIter
            self.render_state.queue.submit([command_buffer1, command_buffer2]);
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
                state.camera_controller.process_mouse(delta.0, delta.1)
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

