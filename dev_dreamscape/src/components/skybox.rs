use bevy_ecs::prelude::*;


#[derive(Component)]
pub struct Skybox {
    pub texture_name: String,
}

impl Skybox {
    pub fn spawn(mut commands: Commands) {
        commands.spawn((
            Skybox {
                texture_name: "skyboxes/planet_atmosphere".to_string()
            }, 
        ));
    }
}




