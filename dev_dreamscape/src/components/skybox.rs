use bevy_ecs::prelude::*;


#[derive(Component)]
pub struct Skybox {
    pub config: String,
}

impl Skybox {
    pub fn spawn(mut commands: Commands) {
        commands.spawn((
            Skybox {
                config: "placeholder".to_string()
            }, 
        ));
    }
}




