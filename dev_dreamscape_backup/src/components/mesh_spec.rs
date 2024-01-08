use bevy_ecs::prelude::*;


pub enum ShaderStage {
    /*
    Color(ColorShader),
    */
    Diffuse,
    PostProcess,
    Skybox,
}

#[derive(Component)]
pub struct MeshSpec {
    meshpath: String,
    shaderstages: Vec<ShaderStage>
}

impl MeshSpec {
    pub fn new(meshpath: String, shaderstages: Vec<ShaderStage>) -> MeshSpec {
        Self {
            meshpath,
            shaderstages
        }
    }
}
