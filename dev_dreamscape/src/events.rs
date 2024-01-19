use crate::device::SurfaceSize;
use winit::event::{MouseButton, VirtualKeyCode};
use bevy_ecs::prelude::*;
use maths::{Vec3f, Mat4f,UnitQuatf};

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


#[derive(Event)]
pub struct CameraSetEvent {
    pub pos: Vec3f,
    pub rot: UnitQuatf,
    pub projection_matrix: Mat4f
}



