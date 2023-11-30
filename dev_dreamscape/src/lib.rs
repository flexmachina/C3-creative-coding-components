mod app;
mod assets;
mod components;
//mod debug_ui;
mod device;
mod events;
mod frame_time;
mod input;
mod logging; 
mod math;
mod mesh;
mod physics_world;
mod render_tags;
mod render_target;
mod shaders;
mod systems;
mod texture;

use crate::systems::*;
use bevy_ecs::prelude::*;
//use crate::app::App;
use crate::app::{run_app};
use crate::assets::Assets;

use winit::event_loop::EventLoop;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

/*
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    let mut world = World::default();
    world.init_resource::<Schedules>();
    // world.init_resource::<State<AppStates>>(); // TODO use states?

    //let event_loop = EventLoop::new();
    //world.insert_non_send_resource(event_loop);


    Schedule::default().add_system(init_app).run(&mut world);
    Schedule::default().add_system(Assets::load).run(&mut world);

    let spawn_scene_schedule = new_spawn_scene_schedule();
    world.add_schedule(spawn_scene_schedule.0, spawn_scene_schedule.1);

    let preupdate_schedule = new_preupdate_schedule();
    world.add_schedule(preupdate_schedule.0, preupdate_schedule.1);

    let update_schedule = new_update_schedule();
    world.add_schedule(update_schedule.0, update_schedule.1);

    let render_schedule = new_render_schedule();
    world.add_schedule(render_schedule.0, render_schedule.1);

    loop {
        world.run_schedule(spawn_scene_schedule.1);
        world.run_schedule(preupdate_schedule.1);
        world.run_schedule(update_schedule.1);
        world.run_schedule(render_schedule.1);

        if !world.resource::<App>().running {
            return;
        }
    }
}
*/


#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    run_app()
}
