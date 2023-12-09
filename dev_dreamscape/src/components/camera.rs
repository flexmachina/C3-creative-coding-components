use crate::math::Mat4;
use crate::render_target::RenderTarget;
use bevy_ecs::prelude::*;
use rapier3d::na;

#[derive(Component)]
pub struct Camera {
    aspect: f32,
    znear: f32,
    zfar: f32,
    fov: f32,
    proj_matrix: Mat4,
    // Tags to render via this camera
    render_tags: u32,
    target: Option<RenderTarget>,
}

impl Camera {
    pub fn new(aspect: f32, render_tags: u32, target: Option<RenderTarget>) -> Self {
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
            render_tags,
            target,
        }
    }

    pub fn target(&self) -> &Option<RenderTarget> {
        &self.target
    }

    pub fn target_mut(&mut self) -> Option<&mut RenderTarget> {
        self.target.as_mut()
    }

    pub fn should_render(&self, tags: u32) -> bool {
        self.render_tags & tags == tags
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
