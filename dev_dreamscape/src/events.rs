use crate::device::SurfaceSize;
use winit::event::{MouseButton, VirtualKeyCode};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct WindowResizeEvent {
    pub new_size: SurfaceSize,
}

#[derive(Event)]
pub struct KeyboardEvent {
    pub code: VirtualKeyCode,
    pub pressed: bool,
}

#[derive(Event)]
pub enum MouseEvent {
    Move(f32, f32),
    Button { button: MouseButton, pressed: bool },
}


#[derive(Event)]
pub struct FrameTimeEvent {
    pub duration: std::time::Duration
}
