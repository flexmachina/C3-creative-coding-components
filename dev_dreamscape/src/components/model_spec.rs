use bevy_ecs::prelude::*;


#[derive(Component)]
pub struct ModelSpec {
    pub modelname: String
}

impl ModelSpec {
    pub fn new(modelname: String) -> ModelSpec {
        Self {
            modelname
        }
    }
}
