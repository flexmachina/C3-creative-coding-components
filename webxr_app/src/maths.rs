#![allow(dead_code)]

use nalgebra as na;

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
