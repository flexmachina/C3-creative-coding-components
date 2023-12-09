use log::warn;
use log::error;


pub fn init_logging() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect(
                "Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
}



pub fn printlog(log_str: &str) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            warn!("{}",log_str);
        } else {
            println!("{}",log_str);
        }
    }
}

