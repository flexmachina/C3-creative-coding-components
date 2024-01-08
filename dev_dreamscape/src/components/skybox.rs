use crate::assets::Assets;
use crate::components::transform::Transform;
use crate::components::{RenderOrder};
use bevy_ecs::prelude::*;


#[derive(Component)]
pub struct SkyboxSpec {
    skyboxpath: String,
}

impl SkyboxSpec {
    pub fn new(skyboxpath: String) -> SkyboxSpec {
        Self {
            skyboxpath
        }
    }
}



#[derive(Component)]
pub struct Skybox;

impl Skybox {
    pub fn spawn(mut commands: Commands, assets: Res<Assets>) {
        let transform = Transform::default();
        let skyboxspec = SkyboxSpec::new(String::from("skybox_bgra.dds"));
        commands.spawn((Skybox, RenderOrder(-100), transform, skyboxspec));
    }
}








