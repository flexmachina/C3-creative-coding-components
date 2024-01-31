mod render;
mod update_input_state;
//mod grab_cursor;
mod schedules;

use crate::device::Device;
use crate::events::{KeyboardEvent, WindowResizeEvent, FrameTimeEvent};
use crate::physics_world::PhysicsWorld;
use crate::app::AppState;
use bevy_ecs::prelude::*;
use winit::event::VirtualKeyCode;

pub use render::{render,prepare_render_pipelines,render_to_texture};
pub use update_input_state::update_input_state;
//pub use grab_cursor::grab_cursor;
pub use schedules::{new_spawn_scene_schedule,new_preupdate_schedule,new_hand_update_schedule,
                    new_camera_update_schedule,new_update_schedule,new_render_schedule};
pub use schedules::{SpawnLabel, PreupdateLabel, UpdateLabel, HandUpdateLabel, CameraUpdateLabel, RenderLabel};
use crate::frame_time::FrameTime;

pub fn resize_device(mut device: ResMut<Device>, mut events: EventReader<WindowResizeEvent>) {
    if let Some(e) = events.iter().last() {
        device.resize(e.new_size.width, e.new_size.height);
    }
}

pub fn escape_on_exit(mut appstate: ResMut<AppState>, mut keyboard_events: EventReader<KeyboardEvent>) {
    if keyboard_events
        .iter()
        .any(|e| e.code == VirtualKeyCode::Escape && e.pressed)
    {
        appstate.running = false;
    }
}

pub fn update_physics(mut physics: ResMut<PhysicsWorld>, frame_time: Res<FrameTime>) {
    physics.update(frame_time.delta);
}


pub fn update_frame_time(appstate: Res<AppState>,
    mut frame_time: ResMut<FrameTime>, mut frametime_events: EventReader<FrameTimeEvent>) {
    
    if appstate.frametime_manual && frametime_events.len() > 0
    {
        for frametime_event in frametime_events.iter() {
            frame_time.update(Some(frametime_event.duration));
        }
    } else {
        frame_time.update(None);
    }
}



