mod app;
mod assets;
//mod components;
//mod debug_ui;
mod device;
mod events;
//mod frame_time;
mod input;
mod math;
//mod mesh;
mod physics_world;
//mod render_tags;
//mod render_target;
//mod shaders;
mod systems;
mod texture;

use crate::systems::*;
use bevy_ecs::prelude::*;
use crate::app::App;
use crate::assets::Assets;


fn main() {
    let mut world = World::default();
    world.init_resource::<Schedules>();
    // world.init_resource::<State<AppStates>>(); // TODO use states?

    Schedule::default().add_system(init_app).run(&mut world);
    Schedule::default().add_system(Assets::load).run(&mut world);

    //let spawn_scene_schedule = new_spawn_scene_schedule();
    //world.add_schedule(spawn_scene_schedule.0, spawn_scene_schedule.1);

    //let preupdate_schedule = new_preupdate_schedule();
    //world.add_schedule(preupdate_schedule.0, preupdate_schedule.1);

    //let update_schedule = new_update_schedule();
    //world.add_schedule(update_schedule.0, update_schedule.1);

    //let render_schedule = new_render_schedule();
    //world.add_schedule(render_schedule.0, render_schedule.1);

    loop {
        //world.run_schedule(spawn_scene_schedule.1);
        //world.run_schedule(preupdate_schedule.1);
        //world.run_schedule(update_schedule.1);
        //world.run_schedule(render_schedule.1);

        if !world.resource::<App>().running {
            return;
        }
    }
}
