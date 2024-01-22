use rapier3d::na;
use bevy_ecs::prelude::*;
#[allow(unused_imports)]
use log::error;
use crate::math::{Mat4, Mat4f};
use crate::components::Transform;


#[rustfmt::skip]
#[allow(dead_code)]
pub const OPENGL_TO_WGPU_MATRIX: Mat4f = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[rustfmt::skip]
pub const FLIPY_MATRIX: Mat4f = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug,Component)]
pub struct Camera {
    perspective: na::Perspective3<f32>,
    webxr: bool
}

impl Camera {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32, webxr: bool) -> Self {
        Self {
            perspective: na::Perspective3::new(width as f32 / height as f32, fovy, znear, zfar),
            webxr
        }
    }
    
    // TODO: pass in transform as a parameter when using ECS
    pub fn view_proj(&self, transform: &Transform) -> Mat4f {
        // Removed premultiply by OPENGL_TO_WGPU_MATRIX as it seems
        // to cause a sliding effect relative to the skybox
        // Note: We don't explicitly need the OPENGL_TO_WGPU_MATRIX, but models centered on (0, 0, 0) will be 
        // halfway inside the clipping area when the camera matrix is identity.
        // OPENGL_TO_WGPU_MATRIX * 
        self.projection_matrix() * self.view_matrix(&transform)
    }

    pub fn view_matrix(&self, transform: &Transform) -> Mat4f {
        transform.matrix().try_inverse().unwrap()
    }

    // Dealing with the WebXR coordinate system needs to be taken with care as there are a few complications that
    // arise from rendering with wgpu directly to a WebGL framebuffer.
    // The WebXR projection matrix needs to be manipulated because in WebXR 
    // the framebuffer has a flipped Y coordinate. We therefore:
    // 1. Pre-mutiply the projection matrix by FLIPY_MATRIX which inverts the y coordinate in clip space. 
    // 2. Invert the triangle winding order to CW (see create_render_pipeline() calls)
    pub fn projection_matrix(&self) -> Mat4f {
        match self.webxr {
            true => FLIPY_MATRIX * self.perspective.as_matrix(),
            false => self.perspective.as_matrix().clone()
        }
    }

    // Perspective3.inverse is faster than general matrix inverse
    pub fn inv_projection_matrix(&self) -> Mat4f {
        match self.webxr {
            true => self.perspective.inverse() * FLIPY_MATRIX, // assumes FLIPY_MATRIX is it's own inverse
            false => self.perspective.inverse().clone()
        }
    }

    // Using in WebXR where the projection matrix is provided directly.
    // rather than decomposed aspect fovy, znear zfar.
    // There's a github discussion about why, but the TLDR is there could 
    // potentially be non-standard projection matrices (e.g. with shear),
    // https://github.com/immersive-web/webxr/issues/461
    #[allow(dead_code)]
    pub fn set_projection_matrix(&mut self, matrix: Mat4f) {
        self.perspective = na::Perspective3::from_matrix_unchecked(matrix);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective.set_aspect(width as f32 / height as f32);
    }
}
