use crate::device::{Device, SurfaceSize};
use crate::events::{KeyboardEvent, MouseEvent, WindowResizeEvent,
                    FrameTimeEvent, CameraSetEvent, HandUpdateEvent};
use crate::frame_time::FrameTime;
use crate::math::{Rect, Vec3f, UnitQuatf, Mat4f};
use crate::input::Input;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

#[cfg(target_arch="wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch="wasm32")]
use crate::xr::WebXRApp;

use winit::window::{WindowBuilder, Window};
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent};

use crate::systems::*;
use crate::assets::Assets;
use crate::components::{Camera, Light, ModelSpec, Player, Skybox, Transform};

use crate::logging::{init_logging, printlog};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Resource)]
pub struct AppState {
    pub running: bool,
    pub webxr: bool,
    pub frametime_manual: bool,
}

pub struct App {
    pub world: World,
}

type EventState = SystemState<(
    EventWriter<'static, WindowResizeEvent>,
    EventWriter<'static, KeyboardEvent>,
    EventWriter<'static, MouseEvent>,
    EventWriter<'static, FrameTimeEvent>,
    EventWriter<'static, HandUpdateEvent>,
    EventWriter<'static, CameraSetEvent>,
)>;

type QueryState = SystemState<(
    Res<'static, Device>,
    Res<'static, Assets>,
    ResMut<'static, Renderers>,
    Query<'static, 'static, (&'static Camera, &'static Transform), With<Player>>,
    Query<'static, 'static, &'static Skybox>,
    Query<'static, 'static, (&'static ModelSpec, &'static Transform)>,
    Query<'static, 'static, (&'static Light, &'static Transform)>,
)>;

type EventLoopState = SystemState<(
    NonSend<'static, Window>,
    ResMut<'static, Input>,
    EventWriter<'static, WindowResizeEvent>,
    EventWriter<'static, KeyboardEvent>,
    EventWriter<'static, MouseEvent>,
)>;

#[derive(Resource)]
struct CachedSystemState {
    event_state: EventState,
    query_state: QueryState
}

impl App {
    pub async fn new(window: Window, webxr: bool) -> Self {

        let mut world = World::default();
        world.init_resource::<Schedules>();

        printlog("running run_app - created world");
        let device = Device::new(&window).await;
        let renderers = Renderers::new(&device, webxr);

        world.insert_resource(device);
        world.insert_resource(renderers);
        world.insert_non_send_resource(window);

        world.insert_resource(AppState {
            running: true,
            webxr: webxr,
            frametime_manual: webxr
        });
        //NOTE not sure if this ok as just init_resource
        world.insert_resource(FrameTime::new());
        world.insert_resource(Input::new());
        world.insert_resource(PhysicsWorld::new());

        // Events
        world.init_resource::<Events<WindowResizeEvent>>();
        world.init_resource::<Events<KeyboardEvent>>();
        world.init_resource::<Events<MouseEvent>>();
        world.init_resource::<Events<FrameTimeEvent>>();
        world.init_resource::<Events<HandUpdateEvent>>();
        world.init_resource::<Events<CameraSetEvent>>();

        let event_state: EventState = SystemState::from_world(&mut world);
        let query_state: QueryState = SystemState::from_world(&mut world);

        world.insert_resource(CachedSystemState {
            event_state,
            query_state
        });

        // Schedules
        let spawn_scene_schedule = new_spawn_scene_schedule(webxr);
        world.add_schedule(spawn_scene_schedule.0, spawn_scene_schedule.1);
        let preupdate_schedule = new_preupdate_schedule();
        world.add_schedule(preupdate_schedule.0, preupdate_schedule.1);
        let update_schedule = new_update_schedule();
        world.add_schedule(update_schedule.0, update_schedule.1);
        let hand_update_schedule = new_hand_update_schedule();
        world.add_schedule(hand_update_schedule.0, hand_update_schedule.1);
        let camera_update_schedule = new_camera_update_schedule();
        world.add_schedule(camera_update_schedule.0, camera_update_schedule.1);
        let render_schedule = new_render_schedule();
        world.add_schedule(render_schedule.0, render_schedule.1);

        Self {
            world,
        }
    }

    pub async fn load_assets(&mut self) {
        printlog("Loading assets outside schedule");
        let assets = Assets::load_and_return(&self.world.resource::<Device>()).await;
        printlog("Done loading assets outside schedule");
        self.world.insert_resource(assets);
    }

    #[allow(dead_code)]
    pub fn device(&self) -> &Device {
        //let (_,device,_,_,_,_,_,_,_,_) = self.world_systemstate_get_mut();
        //&device
        self.world.resource::<Device>()
    }

    #[allow(dead_code)]
    pub fn color_format(&self) -> wgpu::TextureFormat {
        //let (_,device,_,_,_,_,_,_,_,_) = self.world_systemstate_get_mut();
        //device.surface_texture_format()
        self.world.resource::<Device>().surface_texture_format()
    }

    #[allow(dead_code)]
    pub fn update_scene(&mut self, duration: std::time::Duration) {        
        self.world.resource_scope(|world, mut cached_state: Mut<CachedSystemState>| {
            let mut frametime_events = cached_state.event_state.get_mut(world).3;
            frametime_events.send(FrameTimeEvent{duration});
        });
        self.world.run_schedule(SpawnLabel);
        self.world.run_schedule(PreupdateLabel);
        self.world.run_schedule(UpdateLabel);
    }

    #[allow(dead_code)]
    pub fn update_hand(
        &mut self,
        hand: bool,
        joint_transforms: Vec<Mat4f>,
        joint_radii: Vec<f32>
    ) {
        self.world.resource_scope(|world, mut cached_state: Mut<CachedSystemState>| {
            let mut hand_update_events = cached_state.event_state.get_mut(world).4;
            hand_update_events.send(HandUpdateEvent {
                hand,
                joint_transforms,
                joint_radii,
            });
        });
        self.world.run_schedule(HandUpdateLabel);
    }

    #[allow(dead_code)]
    pub fn update_camera(&mut self, pos: Vec3f, rot: UnitQuatf, projection_matrix: Mat4f) {
        self.world.resource_scope(|world, mut cached_state: Mut<CachedSystemState>| {
            let mut cameraset_events = cached_state.event_state.get_mut(world).5;                    
            cameraset_events.send(CameraSetEvent {
                pos,
                rot,
                projection_matrix
            });
        });
        self.world.run_schedule(CameraUpdateLabel);
    }

    #[allow(dead_code)]
    pub fn render_to_texture(&mut self, color_texture: &wgpu::Texture, viewport: Option<Rect>, clear: bool) {
        self.world.resource_scope(|world, mut cached_state: Mut<CachedSystemState>| {
            let (device,
                 assets,
                 renderers,
                 camera_qry,
                 skybox_qry,
                 meshes_qry,
                 light_qry
            ) = cached_state.query_state.get_mut(world);   
        
            render_to_texture(
                &device,
                assets,
                renderers,
                camera_qry,
                skybox_qry,
                meshes_qry,
                light_qry,
                &color_texture,
                viewport,
                clear
            );
        });
    }

}




pub struct Experience {
    pub app: Rc<RefCell<App>>,
    #[cfg(target_arch = "wasm32")]
    #[allow(dead_code)]
    xr_app: Option<WebXRApp>,
}

impl Experience {
    async fn new(window: Window, webxr: bool) -> Self {
        let mut app = App::new(window, webxr).await;
        #[cfg(target_arch = "wasm32")]
        {
            if webxr {
                // Ensure WebXRApp is created before
                // loading assets, which currently takes several seconds. 
                // This is so the XrSession is requested as soon as possible after
                // the user interaction that triggers the wasm to load.
                // If there is more than a few second delay, a Security error occurs.
                let xr_app = WebXRApp::new().await;
                app.load_assets().await;
                let app = Rc::new(RefCell::new(app));
                xr_app.start(app.clone());
                Self{app, xr_app: Some(xr_app)}    
            } else {
                app.load_assets().await;
                let app = Rc::new(RefCell::new(app));    
                Self{app, xr_app: None}
            }
        }    
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.load_assets().await;
            let app = Rc::new(RefCell::new(app));
            Self{app}
        }        
    }
}

#[cfg(target_arch = "wasm32")]
fn setup_window_canvas(window: &Window, surface_size: SurfaceSize) {

    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    window.set_inner_size(surface_size);

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("dreamscape")?;
            let canvas = web_sys::Element::from(window.canvas());
            canvas.set_id("canvas");
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

pub async fn run_experience(webxr: bool) {

    init_logging();
    printlog("running run_app - starting");

    let event_loop = EventLoop::new();
    printlog("running init_app - created event_loop");

    let surface_size = SurfaceSize::new(1900, 1200);

    let window = WindowBuilder::new()
        .with_title("Dreamscape")
        .with_inner_size(surface_size)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        setup_window_canvas(&window, surface_size);
    }
    printlog("running init_app - created window");

    let experience = Experience::new(window, webxr).await;
    printlog("running init_app - created experience");

    let mut eventloop_state: EventLoopState = SystemState::from_world(&mut experience.app.borrow_mut().world);

    let event_handler = move |event: Event<()> , _: &EventLoopWindowTarget<()>, 
                             control_flow: &mut ControlFlow| {

        //Not sure if this will work on WASM
        //*control_flow = ControlFlow::Poll;
        let mut app = experience.app.borrow_mut();

        let (
            window,
            mut input,
            mut resize_events,
            mut keyboard_events,
            mut mouse_events,
        ) = eventloop_state.get_mut(&mut app.world);

        input.reset();
        match event {
            /*
            Event::MainEventsCleared => {
                *control_flow = ControlFlow::Exit;
            }
            */
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => mouse_events.send(MouseEvent::Move(delta.0 as f32, delta.1 as f32)),

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    mouse_events.send(MouseEvent::Button {
                        button: *button,
                        pressed: *state == ElementState::Pressed,
                    });
                }

                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: key_state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    keyboard_events.send(KeyboardEvent {
                        code: *keycode,
                        pressed: *key_state == ElementState::Pressed,
                    });
                }

                WindowEvent::Resized(new_size) => {
                    if webxr {
                        return;
                    }    
                    resize_events.send(WindowResizeEvent {
                        new_size: *new_size,
                    });
                }

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    if webxr {
                        return;
                    }    
                    resize_events.send(WindowResizeEvent {
                        new_size: **new_inner_size,
                    });
                }

                _ => (),
            },


            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if webxr {
                    return;
                }
                app.world.run_schedule(SpawnLabel);
                app.world.run_schedule(PreupdateLabel);
                app.world.run_schedule(UpdateLabel);
                app.world.run_schedule(RenderLabel);
            },

            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                if webxr {
                    return;
                }
                window.request_redraw();
            },

            _ => {}
        }

        if !app.world.resource::<AppState>().running {
            *control_flow = ControlFlow::Exit;
            //return;
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





