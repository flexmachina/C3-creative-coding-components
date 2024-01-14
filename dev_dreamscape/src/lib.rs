mod app;
mod assets;
mod components;
mod device;
mod events;
mod frame_time;
mod input;
mod logging; 
mod math;
mod mesh;
mod physics_world;
mod systems;
mod model;
mod texture;
mod wgpu_ext;
mod renderers;

use crate::app::{run_app};

use winit::event_loop::EventLoop;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    run_app().await
}
