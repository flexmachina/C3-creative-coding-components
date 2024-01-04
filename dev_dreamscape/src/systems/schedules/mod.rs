mod spawn_scene;
mod before_update;
mod update;
mod render;
mod global_render;

pub use spawn_scene::new_spawn_scene_schedule;
pub use before_update::new_preupdate_schedule;
pub use update::new_update_schedule;
pub use render::new_render_schedule;


pub use global_render::new_global_render_schedule;
