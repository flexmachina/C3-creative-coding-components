use bevy_ecs::prelude::*;
use crate::components::Transform;

use crate::assets::Assets;
use crate::math::Vec3f;

use super::ModelSpec;


#[derive(Debug,Component)]
pub struct Light {
    pub color: Vec3f,
}

impl Light {
    pub fn spawn(mut commands: Commands, assets: Res<Assets>) {
        //TODO: us .obj withot material? We only need the geometry
        let modelspec = ModelSpec::new(String::from("cube.obj"));
        commands.spawn((
            Light {
                color: Vec3f::new(1., 1., 1.)
            },
            Transform::from_position(Vec3f::new(0., 10., 0.)),
            modelspec
        ));
    }
}
