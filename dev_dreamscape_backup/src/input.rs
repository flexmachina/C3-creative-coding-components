use bevy_ecs::prelude::*;
use winit::event::*;

#[derive(Resource)]
pub struct Input {
    pub lmb_down: bool,
    pub rmb_down: bool,
    pub forward_down: bool,
    pub back_down: bool,
    pub left_down: bool,
    pub right_down: bool,
    pub up_down: bool,
    pub down_down: bool,
    pub space_just_pressed: bool,
    pub mouse_delta: (f32, f32),
}

impl Input {
    pub fn new() -> Self {
        Input {
            lmb_down: false,
            rmb_down: false,
            forward_down: false,
            back_down: false,
            left_down: false,
            right_down: false,
            up_down: false,
            down_down: false,
            space_just_pressed: false,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn reset(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.space_just_pressed = false;
    }

    pub fn on_mouse_move(&mut self, delta: (f32, f32)) {
        self.mouse_delta = delta;
    }

    pub fn on_mouse_button(&mut self, btn: MouseButton, pressed: bool) {
        if btn == MouseButton::Left { self.lmb_down = pressed; }
        if btn == MouseButton::Right { self.rmb_down = pressed; }
    }

    pub fn on_key(&mut self, code: VirtualKeyCode, pressed: bool) {
        match code {
            VirtualKeyCode::W => self.forward_down = pressed,
            VirtualKeyCode::A => self.left_down = pressed,
            VirtualKeyCode::S => self.back_down = pressed,
            VirtualKeyCode::D => self.right_down = pressed,
            VirtualKeyCode::E => self.up_down = pressed,
            VirtualKeyCode::Q => self.down_down = pressed,
            VirtualKeyCode::Space => self.space_just_pressed = pressed,
            _ => (),
        }
    }
}
