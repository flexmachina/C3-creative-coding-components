#![allow(dead_code)]

use std::ops::Add;
use rapier3d::na;
use rapier3d::prelude::Real;

// Type defs for convenience
// Makes it easier to switch the maths library

// Float specializations
pub type Vec2f = na::Vector2<f32>;
pub type Vec3f = na::Vector3<f32>;
pub type Point3f = na::Point3<f32>;
pub type Mat3f = na::Matrix3<f32>;
pub type Mat4f = na::Matrix4<f32>;
pub type Quatf = na::Quaternion<f32>;
pub type UnitQuatf = na::UnitQuaternion<f32>;
pub type UnitVec3f = na::UnitVector3<f32>;

// Generic
pub type Vec2<T> = na::Vector2<T>;
pub type Vec3<T> = na::Vector3<T>;
pub type Point3<T> = na::Point3<T>;
pub type Mat3<T> = na::Matrix3<T>;
pub type Mat4<T> = na::Matrix4<T>;
pub type Quat<T> = na::Quaternion<T>;
pub type UnitQuat<T> = na::UnitQuaternion<T>;
pub type UnitVec3<T> = na::UnitVector3<T>;


// WTF, how else to cast?
pub fn to_point(v3: Vec3f) -> Point3<Real> {
    Point3::origin().add(v3)
}

pub struct Rect {
   pub x: f32,
   pub y: f32,
   pub w: f32,
   pub h: f32
}


pub fn sRGBtolinearRGB(color: Vec3f)-> Vec3f {
    let color_convert = |val: f32| -> f32 {
        (val/255.).powf(2.2)
    };
    Vec3f::new(color_convert(color[0]),
               color_convert(color[1]),
               color_convert(color[2]))
}





