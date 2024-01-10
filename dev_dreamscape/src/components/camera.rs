use crate::math::Mat4;
use bevy_ecs::prelude::*;
use rapier3d::na;

#[derive(Component)]
pub struct Camera {
    aspect: f32,
    znear: f32,
    zfar: f32,
    fov: f32,
    proj_matrix: Mat4,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        let znear = 0.1;
        let zfar = 100.0;
        let fov = 45.0;
        let proj_matrix = na::Perspective3::new(aspect, fov, znear, zfar).to_homogeneous();

        Self {
            aspect,
            znear,
            zfar,
            fov,
            proj_matrix,
        }
    }

    pub fn proj_matrix(&self) -> Mat4 {
        self.proj_matrix
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.proj_matrix =
            na::Perspective3::new(self.aspect, self.fov, self.znear, self.zfar).to_homogeneous();
    }
}
