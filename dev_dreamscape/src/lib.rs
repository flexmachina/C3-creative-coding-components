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
mod xr;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {

    const XR_MODE: bool = true;
    crate::app::run_experience(XR_MODE).await

    /*
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
    */

}
