use crate::components::transform::Transform;
use crate::components::Camera;
use crate::device::Device;
use crate::mesh::{DrawMesh, Mesh};
use crate::shaders::{
    //ColorShader, 
    DiffuseShader, 
    PostProcessShader, 
    SkyboxShader
};
use bevy_ecs::prelude::*;

pub enum ShaderVariant {
    /*
    Color(ColorShader),
    */
    Diffuse(DiffuseShader),
    PostProcess(PostProcessShader),
    Skybox(SkyboxShader),
}

#[derive(Component)]
pub struct MeshRenderer {
    mesh: Mesh,
    shader: ShaderVariant,
    tags: u32, // TODO As a component?
}

impl MeshRenderer {
    pub fn new(mesh: Mesh, shader: ShaderVariant, tags: u32) -> MeshRenderer {
        Self {
            mesh,
            shader,
            tags
        }
    }

    pub fn render<'a>(
        &'a mut self,
        device: &Device,
        camera: (&Camera, &Transform),
        transform: &Transform,
        encoder: &mut wgpu::RenderBundleEncoder<'a>,
    ) {
        // TODO Generalize
        match self.shader {
            /*
            ShaderVariant::Color(ref mut color) => {
                color.update_uniforms(device, camera, transform);
                color.apply(encoder);
            }
            */
            ShaderVariant::Diffuse(ref mut diffuse) => {
                diffuse.update_uniforms(device, camera, transform);
                diffuse.apply(encoder);
            }
            ShaderVariant::Skybox(ref mut skybox) => {
                skybox.update_uniforms(device, camera);
                skybox.apply(encoder);
            }
            ShaderVariant::PostProcess(ref mut pp) => {
                pp.apply(encoder);
            }
        }
        encoder.draw_mesh(&self.mesh);
    }

    pub fn tags(&self) -> u32 {
        self.tags
    }

    pub fn set_tags(&mut self, tags: u32) {
        self.tags = tags;
    }

    pub fn set_shader(&mut self, shader: ShaderVariant) {
        self.shader = shader;
    }
}
