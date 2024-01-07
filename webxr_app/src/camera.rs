use nalgebra::{clamp, Matrix4, Perspective3, Point3, Vector3, UnitQuaternion};
#[allow(unused_imports)]
use log::error;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;


#[rustfmt::skip]
#[allow(dead_code)]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[rustfmt::skip]
pub const FLIPY_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: f32,
    pitch: f32,
    pub projection: Projection,

    // Hack to avoid passing around 2 cameras
    // Ideally we should store the rotation and projection matrices in the same form
    // and convert to the appropriate coordinate system for rendering
    pub xr_camera: XrCamera,
}

impl Camera {
    pub fn new<P: Into<Point3<f32>>>(
        position: P,
        yaw: f32,
        pitch: f32,
        projection: Projection 
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.to_radians(),
            pitch: pitch.to_radians(),
            projection,
            xr_camera: XrCamera { 
                position: [0.0, 0.0, 0.0].into(),
                rotation: UnitQuaternion::identity(),
                projection: Matrix4::identity()
            }
        }
    }

    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_position: self.position.to_homogeneous().into(),
            view_proj: self.view_proj().into()
        }
    }

    pub fn view_proj(&self) -> Matrix4<f32> {
        // Removed premultiply by OPENGL_TO_WGPU_MATRIX as it seems
        // to cause a sliding effect relative to the skybox
        // Note: We don't explicitly need the OPENGL_TO_WGPU_MATRIX, but models centered on (0, 0, 0) will be 
        // halfway inside the clipping area when the camera matrix is identity.
        // OPENGL_TO_WGPU_MATRIX * 
        self.projection.matrix() * self.calc_matrix(self.position)
    }

    pub fn view_proj_skybox(&self) -> Matrix4<f32> {
        // Removed premultiply by OPENGL_TO_WGPU_MATRIX as it messes up the
        // skybox rendering.
        self.projection.matrix() * self.calc_matrix(Point3::origin())
    }

    fn calc_matrix(&self, position: Point3<f32>) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let dir = Vector3::new(cos_pitch * sin_yaw, sin_pitch, cos_pitch * cos_yaw).normalize();
        Matrix4::look_at_lh(
            &position,
            &(position + dir),
            &Vector3::y_axis(),
        )
    }
}

#[derive(Debug)]
pub struct XrCamera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub projection: Matrix4<f32>,
}

impl XrCamera {
    pub fn to_uniform(&self) -> CameraUniform {
        // TODO verify this is correct
        let pos = self.position * -1.;

        CameraUniform {
            view_position: pos.to_homogeneous().into(),
            view_proj: self.view_proj().into()
        }
    }

    pub fn view_proj_skybox(&self) -> Matrix4<f32> {
        // See below for explanation of the maths
        let rot = self.rotation.conjugate();
        let view = Matrix4::from(rot.to_rotation_matrix());
        FLIPY_MATRIX * self.projection * view
    }

    pub fn view_proj(&self) -> Matrix4<f32> {
        // Dealing with the WebXR coordinate system needs to be taken with care as there are a few complications that
        // arise from performating rendering with wgpu directly to a WebGL framebuffer.
        // The WebXR projection matrix, position and orientation memebrs of this struct need to be manipulated because in WebXR 
        // the framebuffer has a flipped Y coordinate. We therefore:
        // 1. Pre-mutiply the projection by FLIPY_MATRIX which inverts the y coordinate in clip space. 
        // 2. Invert the triangle winding order to CW (see create_render_pipeline())
        // 3. Invert the rotation directions to account for the inverted Y
        // 4. Invert the position - not sure if this is related to flipping Y, but it seems necessary
        let pos = self.position * -1.;
        let rot = self.rotation.conjugate();
        // TODO: find better way to convert from Position3 to Vector3
        let pos = Vector3::new(pos.x, pos.y, pos.z);
        let view = Matrix4::from(rot.to_rotation_matrix()) * Matrix4::new_translation(&pos);
        FLIPY_MATRIX * self.projection * view
    }
}

#[derive(Debug)]
pub struct Projection {
    perspective: Perspective3<f32>
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            perspective: Perspective3::new(width as f32 / height as f32, fovy, znear, zfar)
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective.set_aspect(width as f32 / height as f32);
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        self.perspective.as_matrix()
    }
}

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
        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vector3::new(-yaw_sin, 0.0, -yaw_cos).normalize();
        let right = Vector3::new(yaw_cos, 0.0, -yaw_sin).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_sin, pitch_sin, pitch_cos * yaw_cos).normalize();
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
        camera.pitch = clamp(camera.pitch, -SAFE_FRAC_PI_2, SAFE_FRAC_PI_2)
    }
}
