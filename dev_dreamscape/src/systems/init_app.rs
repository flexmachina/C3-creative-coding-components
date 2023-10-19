//use crate::debug_ui::DebugUI;
use crate::device::{Device, SurfaceSize};
use crate::events::{KeyboardEvent, MouseEvent, WindowResizeEvent};
use crate::frame_time::FrameTime;
use crate::input::Input;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::app::App;

pub fn init_app(world: &mut World) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Demo")
        .with_inner_size(SurfaceSize::new(1900, 1200))
        .build(&event_loop)
        .unwrap();

    let device = pollster::block_on(async {
        Device::new(&window).await
    });

    world.init_resource::<Events<WindowResizeEvent>>();
    world.init_resource::<Events<KeyboardEvent>>();
    world.init_resource::<Events<MouseEvent>>();

    world.insert_non_send_resource(event_loop);
    //world.insert_non_send_resource(DebugUI::new(&device, &window));
    world.insert_non_send_resource(window);

    world.insert_resource(App {
        running: true,
    });
    world.insert_resource(device);
    world.insert_resource(FrameTime::new());
    world.insert_resource(Input::new());
    world.insert_resource(PhysicsWorld::new());
}
