use nalgebra::{Matrix4, Perspective3, Point3, Vector3, UnitQuaternion};
#[allow(unused_imports)]
use log::error;


#[rustfmt::skip]
#[allow(dead_code)]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[rustfmt::skip]
pub const FLIPY_MATRIX: Matrix4<f32> = Matrix4::new(
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
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
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
