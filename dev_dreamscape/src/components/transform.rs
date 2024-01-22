#![allow(dead_code)]

use rapier3d::na;
use bevy_ecs::prelude::*;
use crate::math::{Mat4, Mat4f, Vec3, Vec3f, Quatf, UnitQuat, UnitQuatf, UnitVec3f};

#[derive(Component,Debug)]
pub struct Transform {
    // Individual components
    pos: Vec3f,
    rot: UnitQuatf,
    scale: Vec3f,
    // Cached transform matrix
    m: Mat4f,
}

// TODO Parent-child relationships
impl Transform {
    pub fn new(pos: Vec3f, rot: UnitQuatf, scale: Vec3f) -> Self {
        let m = Mat4::identity();
        let mut res = Self { pos, rot, scale, m };
        res.rebuild_matrix();
        res
    }

    pub fn from_position(pos: Vec3f) -> Self {
        Transform::new(pos, UnitQuat::identity(), Vec3::from_element(1.0))
    }

    // Getters

    pub fn matrix(&self) -> Mat4f {
        self.m
    }

    pub fn forward(&self) -> Vec3f {
        -self.m.column(2).xyz()
    }

    pub fn right(&self) -> Vec3f {
        self.m.column(0).xyz()
    }

    pub fn up(&self) -> Vec3f {
        self.m.column(1).xyz()
    }

    pub fn position(&self) -> Vec3f {
        self.pos
    }

    pub fn rotation(&self) -> UnitQuatf {
        self.rot
    }

    pub fn scale(&self) -> Vec3f {
        self.scale
    }

    // Setters

    pub fn look_at(&mut self, target: Vec3f) {
        self.rot = UnitQuat::face_towards(&(target - self.pos), &Vec3::y_axis()); 
        self.rebuild_matrix();
    }

    pub fn translate(&mut self, v: Vec3f) {
        self.m.append_translation_mut(&v);
        self.pos += v;
    }

    pub fn set_position(&mut self, pos: Vec3f) {
        self.pos = pos;
        self.rebuild_matrix();
    }

    pub fn set_pose(&mut self, pos: Vec3f, rot: UnitQuatf) {
        self.pos = pos;
        self.rot = rot;
        self.rebuild_matrix();
    }

    pub fn set_scale(&mut self, scale: Vec3f) {
        self.scale = scale;
        self.rebuild_matrix();
    }

    pub fn set(&mut self, pos: Vec3f, rotation: Quatf) {
        self.rot = UnitQuat::from_quaternion(rotation);
        self.pos = pos;
        self.rebuild_matrix();
    }

    // Rotate by the given rotation.
    #[inline]
    pub fn rotate(&mut self, rotation: UnitQuatf) {
        self.rot = rotation * self.rot;
        self.rebuild_matrix();
    }

    // Rotate relative to the current rotation.
    #[inline]
    pub fn rotate_local(&mut self, rotation: UnitQuatf) {
        self.rot *= rotation;
        self.rebuild_matrix();
    }
    
    // Rotate around the given global `axis` by `angle` (in radians).
    #[inline]
    pub fn rotate_axis(&mut self, axis: &UnitVec3f, angle: f32) {
        self.rotate(UnitQuat::from_axis_angle(axis, angle));
        // self.rotate calls rebuild_matrix
    }

    // Rotate the local `axis` by `angle` (in radians).
    #[inline]
    pub fn rotate_local_axis(&mut self, axis: &UnitVec3f, angle: f32) {
        self.rotate_local(UnitQuat::from_axis_angle(axis, angle));
        // self.rotate calls rebuild_matrix
    }

    fn rebuild_matrix(&mut self) {
        let rot_m = na::Rotation3::from(self.rot);
        let tr_m = na::Translation3::new(self.pos.x, self.pos.y, self.pos.z);
        let rot_and_tr_m = tr_m * rot_m;
        self.m = rot_and_tr_m
            .to_matrix()
            .prepend_nonuniform_scaling(&self.scale);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new(Vec3::new(0.0, 0.0, 0.0), UnitQuat::identity(), Vec3::from_element(1.0))
    }
}
