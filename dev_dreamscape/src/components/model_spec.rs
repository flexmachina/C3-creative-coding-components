use bevy_ecs::prelude::*;


pub enum ShaderStage {
    /*
    Color(ColorShader),
    */
    Diffuse,
    Skybox,
}

#[derive(Component)]
pub struct ModelSpec {
    pub modelname: String,
    pub shaderstages: Vec<ShaderStage>
}

impl ModelSpec {
    pub fn new(modelname: String, shaderstages: Vec<ShaderStage>) -> ModelSpec {
        Self {
            modelname,
            shaderstages
        }
    }
}
