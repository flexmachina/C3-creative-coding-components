use bevy_ecs::prelude::*;

use crate::{model};

pub enum ShaderStage {
    /*
    Color(ColorShader),
    */
    Diffuse,
    Skybox,
}

#[derive(Component)]
pub struct ModelSpec {
    modelname: String,
    shaderstages: Vec<ShaderStage>
}

impl ModelSpec {
    pub fn new(modelname: String, shaderstages: Vec<ShaderStage>) -> ModelSpec {
        Self {
            modelname,
            shaderstages
        }
    }
}
