use std::{time::Duration, f32::consts::PI};
use std::f32::consts::FRAC_PI_2;

use winit::dpi::PhysicalPosition;
use winit::event::*;

use crate::{
    camera::Camera,
    transform::Transform,
    maths::{Vec3, Vec3f}
};

#[allow(dead_code)]
const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,

    // Adding this as a hack to match player.rs in dev_dreamscape app
    translation_acc: Vec3f,
    v_rot_acc: f32,
    h_rot_acc: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            translation_acc: Vec3::zeros(),
            v_rot_acc: 0.0,
            h_rot_acc: 0.0,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        self.rotate(&mut camera.transform, dt);
        self.translate(&mut camera.transform, dt);
    }

    /*
    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();
        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(-yaw_sin, 0.0, -yaw_cos).normalize();
        let right: na::Matrix<f32, na::Const<3>, na::Const<1>, na::ArrayStorage<f32, 3, 1>> = Vec3::new(yaw_cos, 0.0, -yaw_sin).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward =
            Vec3::new(pitch_cos * yaw_sin, pitch_sin, pitch_cos * yaw_cos).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += (-self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += (self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        camera.pitch = na::clamp(camera.pitch, -SAFE_FRAC_PI_2, SAFE_FRAC_PI_2)
    }
    */

    fn translate(&mut self, transform: &mut Transform, dt: f32) {
        let mut translation = Vec3f::from_element(0.0);
        translation += transform.forward() * self.amount_forward;
        translation -= transform.forward() * self.amount_backward;
        translation += transform.right() * self.amount_right;
        translation -= transform.right() * self.amount_left;
        translation += transform.up() * self.amount_up;
        translation -= transform.up() * self.amount_down;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        translation += transform.forward() * self.scroll * self.sensitivity;
        self.scroll = 0.0;

        // Apply only if there's anything to apply. Otherwise getting NaN after normalize() :|
        // TODO: use try_normalize()
        if translation.magnitude() > 0.01 {
            self.translation_acc += translation.normalize() * self.speed * dt;
        }

        // let (possible_translation, collider_current_pos) = physics
        //     .move_character(dt, self.translation_acc, self.collider_handle);
        let possible_translation = translation;
        self.translation_acc = possible_translation;

        let translation = self.speed * dt * self.translation_acc;
        self.translation_acc -= translation;

        transform.translate(translation);
    }

    fn rotate(&mut self, transform: &mut Transform, dt: f32) {
        const MIN_TOP_ANGLE: f32 = 0.1;
        const MIN_BOTTOM_ANGLE: f32 = PI - 0.1;

        let angle_to_top = transform.forward().angle(&Vec3::y_axis());

        self.v_rot_acc += self.rotate_vertical * self.sensitivity * dt;

        log::error!("rotate_vertical: {}", self.rotate_vertical);
        log::error!("rotate_horizontal: {}", self.rotate_horizontal);

        // // Protect from overturning - prevent camera from reaching the vertical line with small
        // // margin angles.
        if self.v_rot_acc + angle_to_top <= MIN_TOP_ANGLE {
            self.v_rot_acc = MIN_TOP_ANGLE - angle_to_top;
        } else if self.v_rot_acc + angle_to_top >= MIN_BOTTOM_ANGLE {
            self.v_rot_acc = MIN_BOTTOM_ANGLE - angle_to_top;
        }

        // Smooth the movement a bit
        let v_rot = self.speed * dt * self.v_rot_acc;
        self.v_rot_acc -= v_rot;

        self.h_rot_acc += self.rotate_horizontal * self.sensitivity * dt;
        let h_rot = self.speed * dt * self.h_rot_acc;
        self.h_rot_acc -= h_rot;

        // The game world uses a right-hand coordinate system (where a positive angle
        // is anti-clockwise), so negate the angles
        transform.rotate_axis(&Vec3::y_axis(), -h_rot);
        transform.rotate_local_axis(&Vec3::x_axis(), -v_rot);

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }
}
