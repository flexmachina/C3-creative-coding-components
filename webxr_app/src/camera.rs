use nalgebra as na;
#[allow(unused_imports)]
use log::error;

use crate::maths::{Mat4, Mat4f, Vec3};
use crate::transform::Transform;


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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct Camera {
    pub transform: Transform,
    pub projection: Projection,
}

impl Camera {
    pub fn new(
        transform: Transform,
        projection: Projection,
    ) -> Self {
        Self {
            transform,
            projection
        }
    }

    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_position: self.transform.position().to_homogeneous().into(),
            view_proj: self.view_proj().into()
        }
    }
    
    // TODO: pass in transform as a parameter when using ECS
    pub fn view_proj(&self) -> Mat4f {
        // Removed premultiply by OPENGL_TO_WGPU_MATRIX as it seems
        // to cause a sliding effect relative to the skybox
        // Note: We don't explicitly need the OPENGL_TO_WGPU_MATRIX, but models centered on (0, 0, 0) will be 
        // halfway inside the clipping area when the camera matrix is identity.
        // OPENGL_TO_WGPU_MATRIX * 
        self.projection.matrix() * self.view_matrix()
    }
    
    // TODO: pass in transform as a parameter when using ECS
    pub fn view_proj_skybox(&self) -> Mat4f {
        // Skybox needs view mat at origin
        let t = Transform::new(Vec3::zeros(), self.transform.rotation(), self.transform.scale());
        // Removed premultiply by OPENGL_TO_WGPU_MATRIX as it messes up the
        // skybox rendering.
        //OPENGL_TO_WGPU_MATRIX *
        self.projection.matrix() * t.matrix().try_inverse().unwrap()
    }

    pub fn view_matrix(&self) -> Mat4f {
        self.transform.matrix().try_inverse().unwrap()
    }
}

#[derive(Debug)]
pub struct Projection {
    perspective: na::Perspective3<f32>,
    webxr: bool
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32, webxr: bool) -> Self {
        Self {
            perspective: na::Perspective3::new(width as f32 / height as f32, fovy, znear, zfar),
            webxr
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective.set_aspect(width as f32 / height as f32);
    }

    // Dealing with the WebXR coordinate system needs to be taken with care as there are a few complications that
    // arise from rendering with wgpu directly to a WebGL framebuffer.
    // The WebXR projection matrix needs to be manipulated because in WebXR 
    // the framebuffer has a flipped Y coordinate. We therefore:
    // 1. Pre-mutiply the projection matrix by FLIPY_MATRIX which inverts the y coordinate in clip space. 
    // 2. Invert the triangle winding order to CW (see create_render_pipeline() calls)
    //
    // Note the other WebXR coordinate system related conversion is reversing the 
    // camera rotation direction in xr.rs
    pub fn matrix(&self) -> Mat4f {
        match self.webxr {
            true => FLIPY_MATRIX * self.perspective.as_matrix(),
            false => self.perspective.as_matrix().clone()
        }
    }

    // Using in WebXR where the projection matrix is provided directly.
    // rather than decomposed aspect fovy, znear zfar.
    // There's a github discussion about why, but the TLDR is there could 
    // potentially be non-standard projection matrices (e.g. with shear),
    // https://github.com/immersive-web/webxr/issues/461
    pub fn set_matrix(&mut self, matrix: Mat4f) {
        self.perspective = na::Perspective3::from_matrix_unchecked(matrix);
    }
}
