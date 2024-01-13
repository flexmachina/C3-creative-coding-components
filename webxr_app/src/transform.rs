use nalgebra as na;

use crate::maths::{Mat4, Mat4f, Vec3, Vec3f, Quatf, UnitQuat, UnitQuatf};


pub enum TransformSpace {
    Local,
    World,
}

#[derive(Debug)]
pub struct Transform {
    // Cached transform matrix
    m: Mat4f,
    // Individual components
    scale: Vec3f,
    pos: Vec3f,
    rot: UnitQuatf,
}

// TODO Parent-child relationships
impl Transform {
    pub fn new(pos: Vec3f, rot: UnitQuatf, scale: Vec3f) -> Self {
        let m = Mat4::identity();
        let mut res = Self { m, rot, scale, pos };
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
        self.rot = UnitQuat::look_at_rh(&(target - self.pos), &Vec3::y_axis());
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

    pub fn rotate_around_axis(&mut self, axis: Vec3f, angle: f32, space: TransformSpace) {
        let axis = axis.normalize();
        let axis: na::Matrix<f32, na::Const<3>, na::Const<1>, na::ArrayStorage<f32, 3, 1>> = match space {
            TransformSpace::Local => axis,
            TransformSpace::World => self.m.try_inverse().unwrap().transform_vector(&axis),
        };

        self.rot = UnitQuat::from_scaled_axis(axis * angle) * self.rot;
        self.rebuild_matrix();
    }

    fn rebuild_matrix(&mut self) {
        let rot_m = na::Rotation3::from(self.rot).transpose();
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
