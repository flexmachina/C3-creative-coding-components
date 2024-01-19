mod app;
mod assets;
mod components;
mod device;
mod events;
mod frame_time;
mod input;
mod logging; 
mod math;
mod physics_world;
mod systems;
mod model;
mod texture;
mod utils;
mod renderers;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    crate::app::run_app().await
}
