use std::ops::Add;
use rapier3d::na;
use rapier3d::na::Point3;
use rapier3d::prelude::Real;

pub type Vec3 = na::Vector3<f32>;
pub type Mat4 = na::Matrix4<f32>;
pub type Quat = na::Quaternion<f32>;
pub type UnitQuat = na::UnitQuaternion<f32>;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

// WTF, how else to cast?
pub fn to_point(v3: Vec3) -> Point3<Real> {
    Point3::origin().add(v3)
}
