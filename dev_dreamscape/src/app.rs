use crate::device::{Device, SurfaceSize};
use crate::events::{KeyboardEvent, MouseEvent, WindowResizeEvent,
                    FrameTimeEvent, CameraSetEvent};
use crate::frame_time::FrameTime;
use crate::input::Input;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

#[cfg(target_arch="wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use winit::window::{WindowBuilder, Window};
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent};

use crate::systems::*;
use crate::assets::{Assets, Renderers};
use crate::xr::{WebXRApp};
use crate::logging::{init_logging, printlog};


#[derive(Resource)]
pub struct AppState {
    pub running: bool,
    pub webxr: bool,
    pub frametime_manual: bool,
}

pub struct App {
    pub world: World,
    pub world_systemstate: Rc<RefCell<SystemState>>,
}


impl App {
    pub async fn new(window: Window, webxr: bool) -> Self {

        let mut world = World::default();
        world.init_resource::<Schedules>();

        printlog("running run_app - created world");
        //init_app(&mut world).await;
        printlog("running run_app - run init_app");

        println!("running init_app - started");
        printlog("running init_app - created logger");

        printlog("running init_app - created window");

        /*
        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            window.set_inner_size(PhysicalSize::new(450, 400));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }
         */

        //let device = pollster::block_on(async {
        //    Device::new(&window).await
        //});
        let device = Device::new(&window).await;

        printlog("running init_app - loading assets outside schedule");
        //let assets = Assets::load_and_return(&device);
        let assets = Assets::load_and_return(&device).await;
        /*
        let assets = pollster::block_on(async {
            Assets::load_and_return(&device).await
        });
        */
        let renderers = Renderers::init(&device);
        printlog("running init_app - done loading assets outside schedule");

        world.init_resource::<Events<WindowResizeEvent>>();
        world.init_resource::<Events<KeyboardEvent>>();
        world.init_resource::<Events<MouseEvent>>();
        world.init_resource::<Events<FrameTimeEvent>>();

        //world.insert_non_send_resource(event_loop);
        world.insert_non_send_resource(window);

        world.insert_resource(AppState {
            running: true,
            webxr: webxr,
            frametime_manual: webxr
        });
        world.insert_resource(device);
        world.insert_resource(assets);
        //NOTE not sure if this ok as just init_resource
        world.insert_resource(renderers);
        world.insert_resource(FrameTime::new());
        world.insert_resource(Input::new());
        world.insert_resource(PhysicsWorld::new());

        let mut world_systemstate: SystemState<(
            NonSend<Window>,
            Res<Device>,
            Res<Assets>,
            Res<Renderers>,
            //NonSendMut<EventLoop<()>>,
            ResMut<Input>,
            EventWriter<WindowResizeEvent>,
            EventWriter<KeyboardEvent>,
            EventWriter<MouseEvent>,
            EventWriter<FrameTimeEvent>,
            EventWriter<CameraSetEvent>,
        )> = SystemState::from_world(&mut world);


        let spawn_scene_schedule = new_spawn_scene_schedule();
        world.add_schedule(spawn_scene_schedule.0, spawn_scene_schedule.1);
        let preupdate_schedule = new_preupdate_schedule();
        world.add_schedule(preupdate_schedule.0, preupdate_schedule.1);
        let update_schedule = new_update_schedule();
        world.add_schedule(update_schedule.0, update_schedule.1);
        let render_schedule = new_render_schedule();
        world.add_schedule(render_schedule.0, render_schedule.1);

        Self {
            world,
            world_systemstate
        }
    }

    fn device(&self) -> &Device {
        let (_,device,_,_,_,_,_,_,_,_) = self.world_systemstate.get(&self.world);
        device
    }

    fn color_format(&self) -> wgpu::TextureFormat {
        let (_,device,_,_,_,_,_,_,_,_) = self.world_systemstate.get(&self.world);
        device.surface_texture_format()
    }

    #[allow(dead_code)]
    fn update_scene(&mut self, duration: std::time::Duration,
                    pos: Vec3f, rot: UnitQuatf, projection_matrix: Mat4f) {
        //TODO need to set the time via event
        let (_,_,_,_,_,_,_,_,mut frametime_events, mut cameraset_events) = 
                            self.world_systemstate.get_mut(&self.world);
        cameraset_events.send(CameraSetEvent {
            pos,
            rot,
            projection_matrix
        });
        frametime_events.send(FrameTimeEvent {
            duration,
        });
        self.world.run_schedule(spawn_scene_schedule.1);
        self.world.run_schedule(preupdate_schedule.1);
        self.world.run_schedule(update_schedule.1);
    }

    fn render_to_texture(&mut self, color_texture: &wgpu::Texture, depth_texture: Option<&wgpu::Texture>,
                         viewport: Option<Rect>, clear: bool) {
        //TODO access renderers and run returned command buffers

        //basically access the skybox and phong renderers and get the command buffers
        /*
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
            (&self.scene.camera, &self.scene.camera_transform),
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
            (&self.scene.camera, &self.scene.camera_transform),
            &self.scene.light,
            &viewport,
            false,
            clear);

            // submit will accept anything that implements IntoIter
            self.render_state.queue.submit([command_buffer1, command_buffer2]);
        */
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
        // App::new uses async code, so we're going to wait for it to finish
        let app = App::new(window, webxr).await;
        let app = Rc::new(RefCell::new(app));
        #[cfg(target_arch = "wasm32")]
        {
            let xr_app = if webxr {
                let xr_app = WebXRApp::new(app.clone());
                xr_app.init().await;
                Some(xr_app)
            } else {
                None
            };
            Self{app, xr_app}
        }        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self{app}
        }
    }
}


pub async fn run_experience(webxr: bool) {

    init_logging();
    printlog("running run_app - starting");

    let event_loop = EventLoop::new();
    printlog("running init_app - created event_loop");

    let window = WindowBuilder::new()
        .with_title("Demo")
        .with_inner_size(SurfaceSize::new(1900, 1200))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    printlog("running init_app - created window");

    let experience = Experience::new(window, webxr).await;
    printlog("running init_app - created experience");

    //Not sure why this was done? Adding event_loop resource and then removing it again
    //I don't think is needed anymore
    /*
    let mut event_loop = world
        .remove_non_send_resource::<EventLoop<()>>()
        .unwrap();
    */

    let event_handler = move |event: Event<()> , _: &EventLoopWindowTarget<()>, 
                             control_flow: &mut ControlFlow| {

        //Not sure if this will work on WASM
        //*control_flow = ControlFlow::Poll;
        let mut app = experience.app.borrow_mut();
        let (
            mut window,
            _,
            _,
            _,
            mut input,
            mut resize_events,
            mut keyboard_events,
            mut mouse_events,
            _,
            _
        ) = app.world_systemstate.get_mut(&mut world);


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
                    resize_events.send(WindowResizeEvent {
                        new_size: *new_size,
                    });
                }

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    resize_events.send(WindowResizeEvent {
                        new_size: **new_inner_size,
                    });
                }

                _ => (),
            },


            Event::RedrawRequested(window_id) if window_id == window.id() => {
                world.run_schedule(spawn_scene_schedule.1);
                world.run_schedule(preupdate_schedule.1);
                world.run_schedule(update_schedule.1);
                world.run_schedule(render_schedule.1);
            },

            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            },

            _ => {}
        }

        if !world.resource::<AppState>().running {
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





