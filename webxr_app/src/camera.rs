use nalgebra as na;
#[allow(unused_imports)]
use log::error;

use crate::maths::{Mat4, Mat4f, Point3, Point3f, Vec3, UnitQuat, UnitQuatf};
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

    // Hack to avoid passing around 2 cameras
    // Ideally we should store the rotation and projection matrices in the same form
    // and convert to the appropriate coordinate system for rendering
    pub xr_camera: XrCamera,
}

impl Camera {
    pub fn new(
        transform: Transform,
        projection: Projection 
    ) -> Self {
        Self {
            transform,
            projection,
            xr_camera: XrCamera { 
                position: [0.0, 0.0, 0.0].into(),
                rotation: UnitQuat::identity(),
                projection: Mat4::identity()
            }
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
        self.projection.matrix() * self.view_matrix(self.transform.position().into())
    }
    
    // TODO: pass in transform as a parameter when using ECS
    pub fn view_proj_skybox(&self) -> Mat4f {
        // Removed premultiply by OPENGL_TO_WGPU_MATRIX as it messes up the
        // skybox rendering.
        self.projection.matrix() * self.view_matrix(Point3::origin())
    }

    // TODO: pass in transform as a parameter when using ECS
    fn view_matrix(&self, position: Point3f) -> Mat4f {
        // Compute view matrix directly rather than inverting transform matrix
        // This is potentially less expensive, but another reason is that for skybox rendering 
        // we need the view matrix with the eye position at the origin
        let dir = -self.transform.forward();
        Mat4::look_at_lh(
            &position,
            &(position + dir),
            &Vec3::y_axis(),
        )
    }
}

#[derive(Debug)]
pub struct XrCamera {
    pub position: Point3f,
    pub rotation: UnitQuatf,
    pub projection: Mat4f,
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

    pub fn view_proj_skybox(&self) -> Mat4f {
        // See below for explanation of the maths
        let rot = self.rotation.conjugate();
        let view = Mat4::from(rot.to_rotation_matrix());
        FLIPY_MATRIX * self.projection * view
    }

    pub fn view_proj(&self) -> Mat4f {
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
        // TODO: find better way to convert from Position3 to Vec3f
        let pos = Vec3::new(pos.x, pos.y, pos.z);
        let view = Mat4::from(rot.to_rotation_matrix()) * Mat4::new_translation(&pos);
        FLIPY_MATRIX * self.projection * view
    }
}

#[derive(Debug)]
pub struct Projection {
    perspective: na::Perspective3<f32>
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            perspective: na::Perspective3::new(width as f32 / height as f32, fovy, znear, zfar)
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.perspective.set_aspect(width as f32 / height as f32);
    }

    pub fn matrix(&self) -> &Mat4f {
        self.perspective.as_matrix()
    }
}
