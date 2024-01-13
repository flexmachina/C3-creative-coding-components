use crate::assets::Assets;
use crate::components::transform::Transform;
use crate::components::{RenderOrder};
use bevy_ecs::prelude::*;

use crate::{texture};


#[derive(Component)]
pub struct Skybox {
    pub config: String,
}

impl Skybox {
    pub fn spawn(mut commands: Commands, assets: Res<Assets>) {
        let transform = Transform::default();
        commands.spawn((RenderOrder(-100), transform, 
                        Skybox {config: "placeholder".to_string()}));
    }
}




